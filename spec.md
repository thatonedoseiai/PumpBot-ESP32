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
 - A "Menu Option" is a full-width "button" where there is text seperated by a line, almost as if a spreadsheet column, that are, by default 320x32 pixels in size, including left aligned, vertically centered "Standard" text. When it is unselectable, it is only drawn, and the user cannot highlight it in any way.
 - A "Tooltip" is a  full-width text box that includes a tip for whatever item is currently highlighted. This will be, by default, 320x48, on the very bottom of the screen, and include left aligned, top justified "Small" text.
 - A "Swipe" transition is a transition that swipes the previous screen, revealing the new screen underneath from left to right, referred to as "Swipe Right", or right to left, referred to as "Swipe Left". This transition slides the previous screen in the desired direction with a "Smoothstep" progression in roughly 0.75s.

### Fonts:
- Unless specified, "Large" font is 32/40px, "Medium" font is 24/32px, "Standard" font is 20/24px, and "Small" text is 16/20px. all sizes are shown in non-CJK/CJK sizes, respectively. Ideally, CJK and non-CJK text will not coexist in the same text field, but if it does, default to the size of the first character.

### Program:
 - Starts inside of VM, main UI/Ux of PumpBot.
 - On first boot, PumpBot will open in a "Welcome!" screen, saying "Welcome to\nPumpBot!" in a Large font, with underneath, on the bottom of the screen, "Press ⤓ To Continue" in "Standard" font. After 5 seconds, that text will change out to other languages in a predetermined order, drawing over the previous text from left to right with a linear progression of about 0.5s. 
 - After ⤓ (ENC-SW) is pressed, the screen will stop updating, then "Swipe Left" into Setup Screen 0.
 - Setup Screen 0
    - "Medium" text on the top, reading "Choose your Language", with a menu option, reading "Language", left justified, and in the same menu, right justified, the language selected. The languages can be scrolled through with the encoder, and selected by pressing the encoder switch. The language of this screen will change whenever the language is selected. On the very bottom of the screen, there will be an unselectable menu option displaying on the far right, "Apply". Pressing SW1 (underneath "Apply") will apply changes and immediately transition to Setup Screen 1.
    If nothing is pressed within 30 seconds, the screen will "Swipe Right" back into the "Welcome!" screen.
 - Setup Screen 1
    - "Medium" text on the top, reading "How would you like to\nsetup PumpBot?", with 2 Menu Options, centered on the screen, reading "Wi-Fi Setup" and "Standalone Setup". In the "Tooltip", "Set up PumpBot by connecting another device" will be shown when "Wi-Fi Setup" is highlighted, and "Set up PumpBot without connecting another device" will be shown when "Standalone Setup" is highlighted.
    If "Wi-Fi Setup" is selected (by pressing SW1), transition immediately to Setup Screen 2. If "Standalone Setup" is selected, "Swipe Left" transition into menu 3. 
 - Setup Screen 2
    - "Medium" text, dead center of the screen, reading "Starting Wi-Fi Network...". An unselectable menu option on the bottom left of the screen will have the option "Cancel". Pressing "Cancel" (with SW0) will bring you back to Setup Screen 1. Once the Wi-Fi network has initialized, the screen will immediately transition to Setup Screen 2a.
 - Setup Screen 2a
    - "Standard" text, "Connect to this Wi-Fi network to configure PumpBot:", with a unselectable menu option underneath with the network name, as well as "Open this URL in your web browser:", with another unselectable menu option underneath with a URL hosted by PumpBot that's used to configure settings. All of this will be roughly centered to the screen. On the very bottom of the screen, there will be an unselectable menu option displaying, on the far left, "Cancel", and on the far right, "Apply". Pressing SW0 (underneath "Cancel") will bring you back to Setup Screen 1, and pressing SW1 (underneath "Apply") will apply changes and immediately transition to Setup Screen 7.
 - Screen 3
   - "Medium" text on the top, reading "Wi-Fi Setup", with "Standard" text reading "Searching..." underneath. As soon as Wi-Fi networks are found, "Searching..." will be replaced by a list of availible networks in menu options. They can be scrolled through with the encoder, and selected by pressing the encoder switch. On the very bottom of the screen, there will be an unselectable menu option displaying, on the far left, "Back", and on the far right, "Next". Pressing SW0 (underneath "Back") will bring you to Setup Screen 1, and pressing SW1 (underneath "Next") will bring you to Setup Screen 4.
 - Screen 4
   - 
 - Screen 5
   - 
 - Screen 6
   - 
 - Screen 7
   - 

    list of setup screens:
    0 - Language
    1 - wi-fi or standalone setup
    2 - connect to PumpBot via wi-fi
    2a - connect to pumpbot
    3 - Wi-Fi login
    4 - server login
    5 - theme, display brightness
    6 - misc (units of pressure, basic rgb settings)
    7 - setup finished
