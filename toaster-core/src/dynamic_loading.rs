use std::fs;

use std::path::{Path, PathBuf};

use std::sync::{
    Arc, Weak,
    atomic::{
        AtomicU32,
        Ordering
    },
};
use std::collections::BTreeMap;

use parking_lot::{
    Mutex,
    RwLock
};

use libloading::{
    Library,
    Symbol,
};

use serenity::prelude::TypeMapKey;
use serenity::framework::standard::CommandGroup;

pub struct GroupLib
{
    pub group: &'static CommandGroup,
    pub lib: Library,
}


type SliceFn = fn() -> &'static [&'static CommandGroup];

pub struct PluginManager
{
    // Path to the library file to load. Must exist
    lib_path: PathBuf,
    // Path to the directory used for temporary file storage. May not exist
    temp_dir: PathBuf,

    // Incremented once per library load for unique suffixes
    lib_load_counter: AtomicU32,

    // Map from group name to its GroupLib wrapper
    group_map: RwLock<BTreeMap<String, Arc<GroupLib>>>,
    // Vec holding libs that were unloaded previously.
    // To be manually cleared on next usage
    unload_buffer: Mutex<Vec<Arc<GroupLib>>>
}

impl TypeMapKey for PluginManager
{
    type Value = Arc<PluginManager>;
}

impl Default for PluginManager
{
    fn default() -> Self
    {
        PluginManager {
            lib_path: Path::new("").to_owned(),
            temp_dir: Path::new("").to_owned(),

            lib_load_counter: AtomicU32::new(0),

            group_map: RwLock::new(BTreeMap::new()),
            unload_buffer: Mutex::new(Vec::new()),
        }
    }
}

impl PluginManager
{
    const GET_SLICE_FN: &'static [u8] = b"get_group_slice\0";

    pub fn new(lib_path: &str, temp_dir: &str) -> Result<Self, String>
    {
        let mut plugin_manager = Self::default();

        plugin_manager.set_lib_path(lib_path)?;
        plugin_manager.set_temp_dir(temp_dir)?;

        Ok(plugin_manager)
    }

    pub fn set_lib_path(&mut self, lib_path: &str) -> Result<(), String>
    {
        let lib_path = Path::new(lib_path).to_owned();

        if !lib_path.exists()
        {
            return Err("[PluginManager::set_lib_path] Lib_path doesn't exist!".to_owned());
        }

        self.lib_path = lib_path;
        Ok(())
    }

    pub fn set_temp_dir(&mut self, temp_dir: &str) -> Result<(), String>
    {
        let temp_dir = Path::new(temp_dir).to_owned();

        if !temp_dir.exists()
        {
            fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("[PluginManager::set_temp_dir] Unable to create temp_dir! Error: '{}'", e))?;
        }
        
        // Don't want to have to deal with whatever weirdness comes from removing the default I've set it to...
        if self.temp_dir != Self::default().temp_dir
        {
            fs::remove_dir(&self.temp_dir)
                .map_err(|e| format!("[PluginManager::set_temp_dir] Unable to remove old temp_dir! Error: '{}'", e))?;
        }

        self.temp_dir = temp_dir;
        Ok(())
    }

    pub fn list_groups(&self) -> Vec<String>
    {
        let read_lock = self.group_map.read();

        // Gets an iterator over the keys, makes it a cloning iterator, then collects it
        read_lock
            .keys()
            .cloned()
            .collect()
    }

    // Loads all groups in the library
    // Does this by grabbing the slice once and using it to enumerate over all the groups
    pub fn load_all_groups(&self) -> Result<Vec<Weak<GroupLib>>, String>
    {
        // This lib is used _only_ for enumerating all the groups in the slice. It gets dropped at the end of the function.
        // Since this is the only place we load the library bare from its direct path, so we shouldn't get conflict from it
        let lib = Library::new(&self.lib_path)
            .map_err(|e| format!("[load_all_groups] Failed to load library! Error: '{}'", e))?;

        // Load the function that gets the group slice then call it to get said slice
        let slice = {
            let get_slice_fn: Symbol<SliceFn> = unsafe { lib.get(Self::GET_SLICE_FN) }
                .map_err(|e| format!("[load_all_groups] Unable to load slice getter fn from library! Error: '{}'", e))?;

            (*get_slice_fn)()
        };

        let mut output = vec![];

        for group in slice
        {
            output.push(self.load_group(group.name)?);
        }

        Ok(output)
    }

    pub fn load_group(&self, group_name: &str) -> Result<Weak<GroupLib>, String>
    {
        // If the group already exists, report an error and exit out
        if self.group_map.read().contains_key(group_name)
        {
            return Err(format!("[PluginManager::load_group] Attempted to load a group that was already loaded! With group: '{}'", group_name));
        }

        println!("Loading group: '{}'", group_name);

        // Creates a formatted unique lib name for the group
        let group_lib_name = self.unique_formatted_group_lib_name(group_name);
        // Uses the lib name to assemble a path for the library
        let group_lib_path = self.temp_dir.join(group_lib_name);

        println!("Starting copy...");

        // Copies lib into temp folder with a unique(ish) name
        fs::copy(&self.lib_path, &group_lib_path)
            .map_err(|e| format!("[PluginManager::load_group] Failure in copying lib for group loading! Error: '{}'", e))?;

        println!("Finished copy! Starting lib loading...");

        let lib = Library::new(&group_lib_path)
            .map_err(|e| format!("[PluginManager::load_group] Failed to load the library! Error: '{}'", e))?;
        
        let group: &'static CommandGroup = {
            // Grabs the function returning the slice of groups, then uses it to get said slice
            let slice: &[&'static CommandGroup] = {
                println!("Getting slice fn!");
                // Unsafe due to type checking not being possible with external libraries
                let get_slice_fn: Symbol<SliceFn> = unsafe { lib.get(Self::GET_SLICE_FN) }
                    .map_err(|e| format!("[PluginManager::load_group] Unable to load slice getter fn from library! Error: '{}'", e))?;

                println!("Getting slice itself...");
                (*get_slice_fn)()
            };


            println!("Searching through slice for group...");
            // Searches the slice for a matching group
            // Then maps the double reference in the Option to a single reference
            let found = slice.into_iter()
                .find(|g| g.name == group_name)
                .map(|g| *g);

            println!("Returning final group!");
                
            // Finally turns it into a result with the given error message and uses trys it to propagate errors
            found.ok_or("[PluginManager::load_group] Unable to find group in library slice!".to_owned())?
        };

        // Library has been loaded and used, the file _should_ be able to be safely removed
        // TODO: make sure removing this now doesn't have repercussions down the road...
        fs::remove_file(&group_lib_path)
            .map_err(|e| format!("[PluginManager::load_group] Unable to remove already loaded library file! Error: '{}'", e))?;

        // Create our final GroupLib object
        let group_lib = Arc::new(GroupLib {
            group,
            lib,
        });

        // Inserts the GroupLib object into the group map, keyed by the group name
        self.group_map.write()
            .insert(String::from(group_name), Arc::clone(&group_lib));

        // Returns a weak pointer to the group lib.
        // The reason for this is because we don't want the Arc held too long and keeping the library loaded
        Ok(Arc::downgrade(&group_lib))
    }

    // Removes a group from the group map, adding it to the unload buffer if it exists then returning it
    pub fn unload_group(&self, group_name: &str) -> Option<Arc<GroupLib>>
    {
        println!("Unloading group: '{}'", group_name);
        let unloaded_group = self.group_map.write().remove(group_name);

        if let Some(group) = &unloaded_group
        {
            self.unload_buffer.lock().push(Arc::clone(group));
        }

        unloaded_group
    }

    pub fn flush_unload_buffer(&self)
    {
        self.unload_buffer.lock().clear();
    }

    // Not sure if this name is too obnoxiously long and I should just deal with a less useful name...
    fn unique_formatted_group_lib_name(&self, group_name: &str) -> String
    {
        // Using atomics for this crap because a mutex is major overkill
        let old = self.lib_load_counter.fetch_add(1, Ordering::SeqCst);
        format!("lib_{}.plugin.{}", group_name, old + 1)
    }
}