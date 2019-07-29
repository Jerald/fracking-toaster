// INSTRUCTIONS:
// To add a new group of commands, create a sub-module with the SAME NAME as the group
// you're making. Then simply add the mod declaration to the macro call below. Ensure
// you've used the group!() macro in your sub-module so there's something to import.
framework_export!{
    mod general;
    mod yolol;
}