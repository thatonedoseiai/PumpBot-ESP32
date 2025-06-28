#include "fontfile.h"
#define FAIL(f, g) printf("ERROR IN %s: code: %d.\n", f, g);
#define SUCCESS(f) printf("%s SUCCESS\n", f);

uint24_RGB background_color;
uint24_RGB foreground_color;
extern FILE* FONT_FILE;
extern char font_file_open;

int TEST_SIZE_CHANGE(void) {
	if(font_file_open) return -3;
	int x = set_font_size(12);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(14);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(18);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(24);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(42);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	return 0;
}

int main(void) {
	background_color.pixelR = 0;
	background_color.pixelG = 0;
	background_color.pixelB = 0;
	foreground_color.pixelR = 0xff;
	foreground_color.pixelG = 0xff;
	foreground_color.pixelB = 0xff;

	int k = TEST_SIZE_CHANGE();
	if(k) {
		FAIL("TEST_SIZE_CHANGE", k);
	} else {
		SUCCESS("TEST_SIZE_CHANGE");
	}
}