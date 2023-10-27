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
### HID notes:
 - SW0 will always be used for leftward operations outside of the home menu (as in, a "back button"), used as on/off in main menu
 - Encoder button will always be used to select a menu item that user can change the value of outside of the home menu, used to switch between what channel you're controlling in main menu. 
 - Encoder will always be CW for increasing a value or moving rightwards, CCW for decreasing a value or moving leftwards.
 - SW1 will be used to navigate into a submenu or for any unhandled usage, used to go into settings in main menu

### Program:
 - Starts inside of VM, main UI/Ux of PumpBot
 - On first boot, PumpBot will open in a "Welcome!" screen, saying "Welcome!" in a large font, with underneath, "Press ⤓ To Continue". After 5 seconds, that text will change out to other languages in a predetermined order, drawing over the previous text from left to right with a linear progression of about 0.5s. 
 - After ⤓ (ENC-SW) is pressed, the screen will stop changing languages, then swipe from right to left with the next screen, with a "Smoothstep" progression. 