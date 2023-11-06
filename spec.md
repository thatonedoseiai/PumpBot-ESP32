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
 - A "Menu Option" is a full-width "button" where there is text seperated by a line, almost as if a spreadsheet column, that are, by default 320x32 pixels in size, including either 1 centered "Standard" text field (called "Centered Menu Option"), or 2 vertically centered "Standard" text fields side by side, one left aligned, the other right aligned (just called "Menu Option"). When it is unselectable, it is only drawn, and the user cannot highlight it in any way.
 - A "Screen Guide" is a "Menu Option", but always rendered at the bottom of the screen, and only 320x24 pixels in size, not including the top line width. By default, when used in the setup screens, it includes 2 vertically centered "Standard" text fields, one left aligned, and the other right aligned. It is never highlightable. The bottom of the screen acts as the line on the bottom of the screen, so only the top line is drawn. Outside of the setup screens, the changes are TBD.
 - A "Tooltip" is a full-width text box that includes a tip for whatever item is currently highlighted. This will be, by default, 320x48, on the very bottom of the screen, and include left aligned, top justified "Small" text.
 - A "Swipe" transition is a transition that swipes the previous screen, revealing the new screen underneath from left to right, referred to as "Swipe Right", or right to left, referred to as "Swipe Left". This transition slides the previous screen in the desired direction with a "Smoothstep" progression in roughly 0.75s.

### Fonts:
- Unless specified, "Large" font is 32/40px, "Medium" font is 24/32px, "Standard" font is 20/24px, and "Small" text is 16/20px. all sizes are shown in non-CJK/CJK sizes, respectively. Ideally, CJK and non-CJK text will not coexist in the same text field, but if it does, default to the size of the first character.

### Program:
 - Starts inside of VM, main UI/UX of PumpBot.
 - On first boot, PumpBot will open in a "Welcome!" screen, saying "Welcome to\nPumpBot!" in a "Large" font, with underneath, on the bottom of the screen, "Press ⤓ To Continue" in "Standard" font. After 5 seconds, that text will change out to other languages in order (TBD, likely EN, ZH, JA, RU, ES, CAN.), drawing over the previous text from left to right with a linear progression of about 0.5s. 
 - After ⤓ (ENC-SW) is pressed, the screen will stop updating, then "Swipe Left" into Setup Screen 0.
 - Setup Screen 0
    - "Medium" text on the top, reading "Choose your Language", with a menu option underneath, reading...  
    `"Language      [Language]"`  
    The languages can be scrolled through with the encoder, and selected by pressing the encoder switch. The language of this screen will change whenever the language is selected. 
    - There will be a "Screen Guide" displaying "Back" on the left, "Apply" on the right. Pressing SW1 (underneath "Apply") will apply changes and immediately transition to Setup Screen 1.
    If nothing is pressed within 30 seconds or SW0 (underneath "Back") is pressed, the screen will "Swipe Right" back into the "Welcome!" screen.
 - Setup Screen 1
    - "Medium" text on the top, reading "How would you like to\nsetup PumpBot?", with 2 "Centered Menu Options", reading...  
    "Wi-Fi Setup"  
    "Standalone Setup"  
    In a "Tooltip", "Set up PumpBot by connecting another device" will be shown when "Wi-Fi Setup" is highlighted, and "Set up PumpBot without connecting another device" will be shown when "Standalone Setup" is highlighted.
    If "Wi-Fi Setup" is selected, transition immediately to Setup Screen 2. If "Standalone Setup" is selected, "Swipe Left" transition into menu 3. 
    - There will be no "Screen Guide".
    - There will be a "Screen Guide" displaying "Back" on the left and "OK" on the right. Pressing SW0 (underneath "Back") will bring you to Setup Screen 0, and pressing SW1 (underneath "OK") will select whatever "Centered Menu Option" is highlighted.
 - Setup Screen 2
    - "Medium" text, dead center of the screen, reading "Starting Wi-Fi Network...".
    - There will be a "Screen Guide" displaying "Cancel" on the left and nothing on the right. Pressing SW0 (underneath "Cancel") will bring you back to Setup Screen 1. Once the Wi-Fi network has initialized, the screen will immediately transition to Setup Screen 2a.
 - Setup Screen 2a
    - "Standard" text, "Connect to this Wi-Fi network to configure PumpBot:", with a unselectable menu option underneath with the network name, as well as "Open this URL in your web browser:", with another unselectable menu option underneath with a URL hosted by PumpBot that's used to configure settings. All of this will be roughly centered to the screen.
    - There will be a "Screen Guide" displaying "Cancel" on the left and "Apply" on the right. Pressing SW0 (underneath "Cancel") will bring you back to Setup Screen 1 after summoning a "Tooltip" reading "Stop Wi-Fi Setup?/n(This may take a few seconds)", requiring the user to press SW0 again before transitioning the screen, and pressing SW1 (underneath "Apply") will apply changes and immediately transition to Setup Screen 7 after summoning a "Tooltip" reading "Stop Wi-Fi Setup and apply all saved changes made on the website?/n(This may take a few seconds)", requiring the user to press SW1 again before transitioning the screen.
 - Screen 3
   - "Medium" text on the top, reading "Wi-Fi Settings", with "Standard" text reading "Searching..." underneath. As soon as Wi-Fi networks are found or the search times out, "Searching..." will be replaced by a list of availible networks, plus a "Manual/Advanced Setup" option, in menu options. They can be scrolled through with the encoder, and selected by pressing the encoder switch. When a Wi-Fi network or "Manual/Advanced Setup" is selected, the screen will immediately transition to Setup Screen 3a.
   - There will be a "Screen Guide" displaying "Back" on the left and "Next" on the right. Pressing SW0 (underneath "Back") will bring you to Setup Screen 1, and pressing SW1 (underneath "Next") will immediately transition to Setup Screen 5.
 - Screen 3a
   - "Medium" text on the top, reading "Wi-Fi Settings", with 4 menu options underneath, reading...  
   `"Network Name        [Network Name]"` (selectable, network name auto-filled if a wi-fi network is selected)  
   `"Password            [Password]"` (selectable)  
   `         "Advanced"`(selectable, another screen for manual/advanced wi-fi setup. Not speccing now)  
   `         "Connect"` (selectable)  
    Selecting the first 2 menu options will bring up a text entry screen, using the rotary encoder to scroll between and select characters. Selecting the 3rd menu option will go into an advanced wi-fi setup screen. These will not be covered here.
    Selecting the 4th menu option will attempt a connection with the current network and provided password, freezing the current screen and changing "Connect" to "Connecting..." and then either to "Connected!" before transitioning to Setup Screen 4 after a 1s delay, or to "Connection Failed!" before changing back to "Connect" after a 1s delay, then unfreezing the screen. 
    - There will be a "Screen Guide" displaying "Back" on the left and "Next" on the right. Pressing SW0 (underneath "Back") will bring you to Setup Screen 3, and pressing SW1 (underneath "Next") will immediately transition to Setup Screen 5, after summoning a "Tooltip" reading "Are you sure? If you aren't connected to Wi-Fi, you won't be able to use any online features of PumpBot.", requiring the user to press SW1 again before transitioning the screen.
 - Screen 4 (visually a clone of Screen 3a)
   - "Medium" text on the top, reading "Server Settings", with 4 menu options underneath, reading...  
   `"Server Address      [Server Address]"` (selectable, auto-filled to default server)  
   `"Password            [Password]"` (selectable)  
   `         "Advanced"`(selectable, Not speccing now)  
   `         "Connect"` 
    Selecting the first 2 menu options will bring up a text entry screen, using the rotary encoder to scroll between and select characters. Selecting the 3rd menu option will go into an advanced server setup screen. These will not be covered here.
    Selecting the 4th menu option will attempt a connection with the current network and provided password, freezing the current screen and changing "Connect" to "Connecting..." and then either to "Connected!" before transitioning to Setup Screen 4 after a 1s delay, or to "Connection Failed!" before changing back to "Connect" after a 1s delay, then unfreezing the screen.
    - There will be a "Screen Guide" displaying "Back" on the left and "Next" on the right. Pressing SW0 (underneath "Back") will bring you to Setup Screen 3, and pressing SW1 (underneath "Next") will immediately transition to Setup Screen 5, after summoning a "Tooltip" reading "Are you sure? If you aren't connected to a server, you won't be able to use any online features of PumpBot.", requiring the user to press SW1 again before transitioning the screen.
 - Screen 5
   - "Medium" text on top, reading "Display Settings", with 2 menu options underneath, reading...  
    `"Brightness  [Brightness]"`  
    `"Theme       [Light/Dark Theme]"`  
    Both menu options are selectable, the first menu option will change the screen brightness when selected(gradation tbd, prolly 0-100%), changing both the number on the right side of the menu and the actual screen brightness in realtime, when you turn the encoder. The 2nd menu option will do the same, but cycle between menu themes and update as fast as possible. No guarantee in speed of that since it'll need to re-render the whole screen. 
    - There will be a "Screen Guide" displaying "Back" on the left and "Next" on the right. Pressing SW0 (underneath "Back") will bring you to Setup Screen 4, and pressing SW1 (underneath "Next") will immediately transition to Setup Screen 6.
 - Screen 6
   - "Medium" text on top, reading "Add-On Settings", with 1 menu option underneath, reading...  
    `"Unit of Pressure   [psi/bar]"`  
    This menu option will cycle between units of pressure, this will be the display pressure of the "MPRLS" sensor.
    - There will be a "Screen Guide" displaying "Back" on the left and "Finish" on the right. Pressing SW0 (underneath "Back") will bring you to Setup Screen 5, and pressing SW1 (underneath "Finish") will immediately transition to Setup Screen 7.
 - Screen 7
   - "Large" text on top, reading "Setup Complete!", with "Press ⤓ To Continue" in "Standard" font on the bottom of the screen, mocking the Welcome screen's layout. After ⤓ is pressed, "Swipe Left" transition to the Home Screen.
   - There will be no "Screen Guide".

    list of setup screens:  
    0 - Language  
    1 - wi-fi or standalone setup  
    2 - connect to PumpBot via wi-fi  
    2a - connect to pumpbot  
    3 - Wi-Fi login  
    3a - Wi-Fi connect  
    4 - server login  
    5 - theme, display brightness  
    6 - add-on settings  
    7 - setup finished  

    Not really sure where to put this but things in regular braces (aka like [This]) are like variables or text strings, not what'd actually be displayed
