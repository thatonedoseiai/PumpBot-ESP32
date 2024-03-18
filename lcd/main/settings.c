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

void set_default_app(char* name) {
	FILE* f = fopen("/mainfs/default_app", "w");
	if(!f)
		ets_printf("ERROR OCCURRED OPENING DEFAULT APP FILE");
	fwrite(ibuf);
	fclose(f);
}

int read_default_app(char* buf, int buflen) {
	FILE* f = fopen("/mainfs/default_app", "r");
	if(!f)
		return 1;
	fread(buf, sizeof(char), buflen, f);
	fclose(f);
	return 0;
}
