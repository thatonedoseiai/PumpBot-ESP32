#include <rom/ets_sys.h>
#include "settings.h"
#include <string.h>

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

void delete_settings_file() {
	remove(SETTINGS_FILE);
}

void set_default_app(char* name) {
	FILE* f = fopen("/mainfs/default_app", "w");
	if(!f)
		ets_printf("ERROR OCCURRED OPENING DEFAULT APP FILE");
	fwrite(name, sizeof(char), strlen(name), f);
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

void set_settings_to_default(SETTINGS_t* settings) {
	memset(&settings->wifi_name, 0, 32);
	memset(&settings->wifi_pass, 0, 64);
	settings->disp_theme = 0;
	settings->disp_brightness = 255;
	for(int i=0;i<4;++i) {
		settings->pwm_min_limit[i] = 0;
		settings->pwm_max_limit[i] = 0x3fff;
		settings->output_set_on_off_only[i] = 0;
		settings->server_ip[i] = 0;
	}
	settings->server_port = 0;
	memset(&settings->server_password, 0, 64);
	settings->pressure_units = PSI;
}
