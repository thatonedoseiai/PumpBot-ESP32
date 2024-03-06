#include <rom/ets_sys.h>
#include "settings.h"

void write_to_file(SETTINGS_t* settings) {
	FILE* sf = fopen(SETTINGS_FILE, "wb");
	fwrite((void*) settings, sizeof(SETTINGS_t), 1, sf);
	fclose(sf);
}

int read_from_file(SETTINGS_t* settings) {
	FILE* sf = fopen(SETTINGS_FILE, "rb");
	if(sf == NULL) {
		ets_printf("failed to open settings file!\n");
		return 1;
	}
	fread((void*) settings, sizeof(SETTINGS_t), 1, sf);
	fclose(sf);
	return 0;
}
