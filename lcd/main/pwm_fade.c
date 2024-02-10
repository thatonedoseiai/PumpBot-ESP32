#include "pwm_fade.h"
#include "oam.h"

void init_pwm_fade_info(pwm_fade_info_t* info, ledc_channel_t channel) {
	info->channel = channel;
	info->cur_step = 0;
	info->max_step = 0;
}

void pwm_setup_fade(pwm_fade_info_t* info, int start, int end, int num_steps) {
	info->start = start;
	info->end = end;
	info->max_step = num_steps;
	info->cur_step = 0;
}

esp_err_t pwm_step_fade(pwm_fade_info_t* info) {
	esp_err_t k = ledc_set_duty(LEDC_LOW_SPEED_MODE, info->channel, (info->cur_step * info->end + (info->max_step - info->cur_step) * info->start) / info->max_step);
	if(k != ESP_OK)
		return k;
	k = ledc_update_duty(LEDC_LOW_SPEED_MODE, info->channel);
	if(k != ESP_OK)
		return k;
	if(info->cur_step < info->max_step)
		info->cur_step++;
	return ESP_OK;
}

void pwm_fade_blocking() {

}
