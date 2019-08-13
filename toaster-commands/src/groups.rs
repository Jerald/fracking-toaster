// INSTRUCTIONS:
// To add a new group of commands, create a sub-module with the SAME NAME as the group
// you're making. Then simply add the mod declaration to the macro call below. Ensure
// you've used the group!() macro in your sub-module so there's something to import.

group_slice_export!{
    mod general;
    mod yolol;
    mod plugins;

    mod frack_you;
}