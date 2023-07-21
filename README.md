## PumpBot ESP-32
firmware for pumpbot. Run on ESP-32. 

Dependencies:
- esp-idf

to build, go to `lcd/` and run `idf.py build flash`

todo list:
- [ ] render OAM to ensure functionality
- [ ] get partial screen rendering to work in ILI driver
- [ ] ensure that font rendering works properly
- [ ] get unicode characters to render
- [ ] draw UI elements (e.g. horizontal/vertical lines)
- [ ] get bitmap sprites loading for non-text graphics
- [ ] get UI animation working