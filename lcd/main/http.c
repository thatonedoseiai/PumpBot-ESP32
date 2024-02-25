#include "http.h"
#include "esp_littlefs.h"
#include "board_config.h"

esp_err_t http_wget(char* url, char* filename) {
    FILE* download = fopen("/mainfs/temp.dl", "ab");
    http_config.url = url;
    http_config.user_data = download;
    esp_http_client_handle_t client = esp_http_client_init(&http_config);
    esp_err_t err = esp_http_client_perform(client);
    fclose(download);
    rename("/mainfs/temp.dl", filename);

    if (err == ESP_OK) {
        int content_length = esp_http_client_get_content_length(client);
        ets_printf("Status = %d, content_length = %d\n",
            esp_http_client_get_status_code(client),
            content_length);
    }
    esp_http_client_cleanup(client);
    return err;
}

esp_err_t HTTP_download_handler(esp_http_client_event_handle_t evt) {
    switch(evt->event_id) {
    case HTTP_EVENT_ON_DATA:
        ets_printf("HTTP_EVENT_ON_DATA, len=%d\n", evt->data_len);
        if(!esp_http_client_is_chunked_response(evt->client)) {
            assert(evt->user_data != NULL);
            fwrite(evt->data, 1, evt->data_len, evt->user_data);
        }
        break;
    case HTTP_EVENT_ON_FINISH:
        ets_printf("HTTP_EVENT FINISH\n");
        // fclose(evt->data);
        break;
    case HTTP_EVENT_DISCONNECTED:
        ets_printf("HTTP_EVENT_DISCONNECTED\n");
        int mbedtls_err = 0;
        esp_err_t err = esp_tls_get_and_clear_last_error(evt->data, &mbedtls_err, NULL);
        if (err != 0) {
            ets_printf("Last esp error code: 0x%x\n", err);
            ets_printf("Last mbedtls failure: 0x%x\n", mbedtls_err);
        }
        // fclose(evt->data);
        break;
    default:
        break;
    }
    return ESP_OK;
}

