# Adding menus

menus have a loop and an jnitialization sequence. They typically also have some kind of state and some arguments (such as the initial text for the text menu)

1. define a structure for its state. Any variables that you wish to use to keep track of something in the menu (like a cursor position or something) should be put inside this struct.

```rs
struct SomeMenuState {
    a_counter: u8,
    lang_counter: u8,
    cursor_position: u16,
    text_mode: char,
}
```

2. If the menu should take some arguments, create a struct that holds those arguments. 

```rs
struct SomeMenuArgs {
    initial_counter_value: u8,
    initial_cursor_position: Option<u16>
}
```

3. Write the initialization code that takes some arguments and initializes the menu. 
    1. If the menu takes no arguments, it suffices to write a simple function in a separate `impl` block to make it. 
    
    ```rs
    impl SomeMenuState {
        pub fn new() -> Self {
            // some screen initialization code or something
            SomeMenuState {
                a_counter: 0,
                lang_counter: 0,
                cursor_position: 0,
                text_mode: 'a',
            }
        }
    }
    ```

    2. If the menu takes some arguments, write the conversion from the arguments into the menu state. 

    ```rs
    impl From<SomeMenuArgs> for SomeMenuState {
        fn from(args: SomeMenuArgs) -> Self {
            // Some initialization code here
            SomeMenuState {
                a_counter: args.initial_counter_value,
                lang_counter: args.initial_counter_value, 
                cursor_position: args.initial_cursor_position, 
                text_mode: 'a',
            }
        }
    }
    ```

4. write the looped code.

```rs
impl MenuBehaviour for SomeMenuState {
    fn update(&self, events: Vec<Event>) -> MenuTransition {
        info!("I am looping yay");
        MenuTransition::None
    }
}
```