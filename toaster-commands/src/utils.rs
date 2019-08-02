// This macro does hacky things. Serenity works by having macros create command groups
// which are turned into public statics with a SCREAMING_SNAKE_CASE name and "_GROUP"
// appended to the group name. The problem this creates is that we'd need to change
// many things whenever we wanted to add a new group. I _very_ much wanted there to
// be a single place you add your new module of commands when you finished them, so
// this macro came into being.
//
// All command groups are sub-modules of this module, so you of course must have
// `mod <module_name>` somewhere in this module for it to exist. The macro piggybacks
// on that and uses the module name to derive the static group variable's name. With
// these names, we can create a single function that attaches all the declared command
// groups and spits out the framework object. Mission complete!
//
// INSTRUCTIONS:
// To add a new group of commands, create a sub-module with the SAME NAME as the group
// you're making. Then simply add the mod declaration to the macro call below. Ensure
// you've used the group!() macro in your sub-module so there's something to import.

macro_rules! framework_export {
    ( $(mod $m:ident;) + ) => {
        $(mod $m;)+
        
        #[no_mangle]
        pub fn framework_factory() -> serenity::framework::standard::StandardFramework
        {
            serenity::framework::standard::StandardFramework::new()
            $(
                .group({ use $m::*; &serenity_group_name::group_name!($m) })
            )+
        }
    }
}

macro_rules! array_export {
    ( $(mod $m:ident;) + ) => {
        // $(mod $m;)+
        
        const ARRAY_LEN: usize = 0 $( + {let _ = stringify!($m); 1 } )+ ;
        static ARRAY_GROUPS: [&serenity::framework::standard::CommandGroup; ARRAY_LEN] = [
            $(
                { use $m::*; &serenity_group_name::group_name!($m) },
            )+
        ];

        pub fn get_group_slice() -> &'static [&'static serenity::framework::standard::CommandGroup]
        {
            &ARRAY_GROUPS
        }
    }
}