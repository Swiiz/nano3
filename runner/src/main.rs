use std::{collections::HashMap, fs, time::Duration};

use anyhow::Result;
use nano_api::Event;
use postcard::to_allocvec;
use wasmtime::*;

fn main() -> Result<()> {
    let engine = Engine::new(Config::new().wasm_threads(true))?;

    let mut modules = HashMap::<String, Module>::new();
    for folder in fs::read_dir("modules")?.filter_map(Result::ok) {
        let path = folder.path();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let module = Module::from_file(&engine, path.join(".wasm"))?;
        modules.insert(name, module);
    }

    type HostState = ();
    let mut store = Store::new(&engine, ());

    let host_hello = Func::wrap(&mut store, |_: Caller<HostState>, param: i32| {
        println!("Got {param} from WebAssembly");
    });

    let memory = SharedMemory::new(
        &engine,
        MemoryTypeBuilder::default()
            .min(17)
            .max(Some(16384)) // TODO: make this configurable
            .shared(true)
            .build()?,
    )?;

    let mut linker = <Linker<HostState>>::new(&engine);
    linker.define(&store, "host", "hello", host_hello)?;
    linker.define(&store, "env", "memory", memory.clone())?;

    let modules = modules
        .into_iter()
        .filter_map(|(name, module)| {
            Some((
                name.clone(),
                linker
                    .instantiate(&mut store, &module)
                    .map_err(|e| {
                        eprintln!("Failed to instantiate module {name}: {e}");
                    })
                    .ok()?,
            ))
        })
        .collect::<HashMap<_, _>>();

    emit_event(
        &modules,
        Event {
            name: "Hello world!".to_string(),
        },
        &mut store,
        &memory,
    )?;

    Ok(())
}

pub fn emit_event<T>(
    modules: &HashMap<String, Instance>,
    event: T,
    mut store: &mut Store<()>,
    memory: &SharedMemory,
) -> Result<()>
where
    T: serde::Serialize,
{
    let bytes = to_allocvec(&event).unwrap();

    let mem_size = memory.data_size();
    if mem_size <= bytes.len() {
        memory.grow((bytes.len() - mem_size) as u64 / 64_000 + 1)?;
    }

    let view = memory.data();
    for (i, byte) in bytes.iter().enumerate() {
        unsafe {
            *view[i + 1].get() = *byte;
        }
    }

    let handlers = modules
        .values()
        .map(|instance| instance.get_typed_func::<u32, ()>(&mut store, "_handle_event"))
        .collect::<Result<Vec<_>, _>>()?;

    for func in handlers {
        let _ = func.call(&mut store, bytes.len() as u32)?;
    }

    Ok(())
}
