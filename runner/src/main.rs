use std::{any::type_name, collections::HashMap, fs, mem::transmute, time::Duration};

use anyhow::Result;
use nano_api::{deserialize, serialize, Event};
use serde::{de::DeserializeOwned, Serialize};
use wasmtime::*;

fn main() -> Result<()> {
    let engine = Engine::default();

    let mut modules = HashMap::<String, Module>::new();
    for folder in fs::read_dir("modules")?.filter_map(Result::ok) {
        let path = folder.path();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let module = Module::from_file(&engine, path.join(".wasm"))?;
        modules.insert(name, module);
    }

    let memory = SharedMemory::new(
        &engine,
        MemoryTypeBuilder::default()
            .min(17)
            .max(Some(16384)) // TODO: make this configurable
            .shared(true)
            .build()?,
    )?;

    let mut store = Store::new(&engine, HostState { memory });

    let hostfn_print = Func::wrap(&mut store, |c: Caller<HostState>, ptr: u32, len: u32| {
        func_from_wasm(&c.data().memory, ptr, len, |msg: String| {
            println!("WASM: {}", msg);
        })
    });

    let mut linker = <Linker<HostState>>::new(&engine);
    linker.define(&store, "host", "print", hostfn_print)?;
    linker.define(&store, "env", "memory", store.data().memory.clone())?;

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
    )?;

    Ok(())
}

pub struct HostState {
    memory: SharedMemory,
}

pub fn func_from_wasm<T: DeserializeOwned>(
    memory: &SharedMemory,
    ptr: u32,
    len: u32,
    f: impl Fn(T),
) {
    let view = &memory.data()[ptr as usize..][..len as usize];
    let slice = unsafe { transmute(view) }; // Safe because UnsafeCell<T> as the same layout as T
    let arg = deserialize(slice).expect("Failed to deserialize argument");
    f(arg);
}

pub fn emit_event<T>(
    modules: &HashMap<String, Instance>,
    event: T,
    mut store: &mut Store<HostState>,
) -> Result<()>
where
    T: serde::Serialize,
{
    let bytes = serialize(&event).unwrap();

    // Because of default 64kb dynamic memory guard
    if bytes.len() >= 64_000 {
        panic!("Too big event: {} {} bytes", type_name::<T>(), bytes.len());
    }

    let memory = &store.data().memory;
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
