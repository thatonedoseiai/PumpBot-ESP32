#ifndef FILE_SERVER_H
#define FILE_SERVER_H

#include "esp_err.h"

esp_err_t example_start_file_server(const char *base_path);
esp_err_t stop_file_server(void);

#endif
