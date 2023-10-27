### ON BOOT:

initialize things:
- display driver
- font renderer
- encoder state machine
- initialize VM
- run program if able
- enter bios menu if SW1 is held or VM exits unsuccessfully

### BIOS MENU:
- text-only, using FreeType
- navigated with encoder and buttons
- menus:
    - choose program
    - test PWM
    - test input
    - reset

#### choose program
- displays list of executables found on filesystem
- can choose one to execute and save as default (to be run by the VM on startup)

#### test PWM
- manually set PWM % and frequency of all LEDs and outputs

#### test input
- display all button statuses
- display count of clicks from rotary encoder
    - rotary encoder clicks decrement on counterclockwise, increment on clockwise

#### reset
- reinitialize the VM, run the program, and return to the BIOS menu if VM exits unsuccessfully

### HID Standardization:
 - SW0 will always be used for leftward operations outside of the home menu (as in, a "back button"), used as on/off in main menu
 - Encoder button will always be used to select a menu item that user can change the value of outside of the home menu, used to switch between what channel you're controlling in main menu. 
 - Encoder will always be CW for increasing a value or moving rightwards, CCW for decreasing a value or moving leftwards.
 - SW1 will be used to navigate into a submenu or for any unhandled usage, used to go into settings in main menu

### Graphics:
 - A "Menu Option" is a full-width "button" where there is text seperated by a line, almost as if a spreadsheet column, that are, by default 320x32 pixels in size, including left aligned, vertically centered "Standard" text.
 - A "Tooltip" is a  full-width text box that includes a tip for whatever item is currently highlighted. This will be, by default, 320x48, on the very bottom of the screen, and include left aligned, top justified "Small" text.
 - A "Swipe" transition is a transition that swipes the previous screen, revealing the new screen underneath from left to right, referred to as "Swipe Right", or right to left, referred to as "Swipe Left". This transition slides the previous screen in the desired direction with a "Smoothstep" progression in roughly 0.75s.

### Fonts:
- Unless specified, "Large" font is 32/40px, "Medium" font is 24/32px, "Standard" font is 20/24px, and "Small" text is 16/20px. all sizes are shown in non-CJK/CJK sizes, respectively. Ideally, CJK and non-CJK text will not coexist in the same text field, but if it does, default to the size of the first character.

### Program:
 - Starts inside of VM, main UI/Ux of PumpBot.
 - On first boot, PumpBot will open in a "Welcome!" screen, saying "Welcome to[\n] PumpBot!" in a Large font, with underneath, on the bottom of the screen, "Press ⤓ To Continue" in "Standard" font. After 5 seconds, that text will change out to other languages in a predetermined order, drawing over the previous text from left to right with a linear progression of about 0.5s. 
 - After ⤓ (ENC-SW) is pressed, the screen will stop updating, then "Swipe Left". The following screens, outlied below, will be setup screens.
 - Setup Screen 1 - Transitioned from Welcome screen w/ Smoothstep
    - Medium sized text on the top, reading "How would you like to setup PumpBot?", with "Standard" text inside of 2 Menu Options, centered on the screen, containing "Wi-Fi Setup" and "Standalone Setup". In the "Tooltip", "Set up PumpBot by connecting another device" will be shown when "Wi-Fi Setup" is highlighted, and "Set up PumpBot without connecting another device" will be shown when "Standalone Setup" is highlighted.
    If "Wi-Fi Setup" is selected (by pressing SW1), transition immediately to Setup Screen 2. If "Standalone Setup" is selected, "Swipe Left" transition into menu 3.

    list of setup screens:
    1 - wi-fi or standalone setup
    2 - connect to PumpBot via wi-fi
    3 - wi-fi login
    3b- server login
    4 - theme, display brightness
    5 - misc (units of pressure, basic rgb settings)
