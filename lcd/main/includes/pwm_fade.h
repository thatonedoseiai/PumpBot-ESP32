#ifndef PWM_FADE_H
#define PWM_FADE_H

#include "oam.h"
#include "driver/ledc.h"

typedef struct pwm_fade_info_t_ {
	ledc_channel_t channel;
	int cur_step;
	int max_step;
	int start;
	int end;
} pwm_fade_info_t;

void init_pwm_fade_info(pwm_fade_info_t* info, ledc_channel_t channel);
void pwm_setup_fade(pwm_fade_info_t* info, int start, int end, int num_steps);
esp_err_t pwm_step_fade(pwm_fade_info_t* info);

#endif
