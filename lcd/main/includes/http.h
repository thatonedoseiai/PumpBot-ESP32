#ifndef HTTP_H
#define HTTP_H

#include "esp_http_client.h"
#include "esp_tls.h"

esp_err_t HTTP_download_handler(esp_http_client_event_handle_t evt);

esp_err_t http_wget(char* url, char* filename);

#endif
