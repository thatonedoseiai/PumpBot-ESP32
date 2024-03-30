#include "pwm_output.h"
#include "settings.h"
#include "rom/ets_sys.h"

pb_output_info_t* pwms;
gptimer_handle_t pwm_irq_timer;
extern SETTINGS_t settings;
extern uint16_t button_disable_counter;
extern volatile atomic_uint_fast16_t rotenc_multiplier_counter;

void update_pwm(int channel) {
	pwms[channel].was_updated = 1;
	if(atomic_load(&pwms[channel].off)) {
		ledc_stop(LEDC_LOW_SPEED_MODE, channel, 0);
		return;
	}
	unsigned int power = output_get_value(channel);
	int range = settings.pwm_max_limit[channel] - settings.pwm_min_limit[channel];
	int offset = settings.pwm_min_limit[channel];
	if(range < 0) {
		range = -range;
		offset = settings.pwm_max_limit[channel];
	}
	if(settings.output_set_on_off_only[channel]) {
		power = 16383;
	}
	if(((power * range) >> 14) < -offset)
		power = 0;
	else
		power = ((power * range) >> 14) + offset;
	if(power > 16383)
		power = 16383;
	ledc_set_duty_and_update(LEDC_LOW_SPEED_MODE, channel, power, 0);
}

static bool pwm_callback(gptimer_handle_t timer_handle, const gptimer_alarm_event_data_t* edata, void* user_ctx) {
	for(int i=0;i<4;++i) {
		uint16_t k = atomic_load(&(pwms[i].cyclesLeft));
		if(k == 1) {
			uint16_t b = atomic_load(&(pwms[i].old_output));
			atomic_store(&(pwms[i].output), b);
			update_pwm(i);
		}
		if(k) {
			atomic_fetch_sub(&(pwms[i].cyclesLeft), 1);
			// ets_printf("channel %d: %d\n", k-1);
		}
	}
	if(button_disable_counter > 0)
		button_disable_counter--;
	uint32_t timer = atomic_load(&rotenc_multiplier_counter);
	if(timer > 0) {
		// atomic_compare_exchange_strong(&time_since_last_input, &timer, timer - 1);
		atomic_fetch_sub(&rotenc_multiplier_counter, 1);
	}
	return true;
}

void init_pb_output_info() {
	pwms = calloc(4, sizeof(pb_output_info_t));
	gptimer_config_t timer_config = {
		.clk_src = GPTIMER_CLK_SRC_DEFAULT, 
		.direction = GPTIMER_COUNT_UP,
		.resolution_hz = 1*1000*1000 // 1MHz
	};
	gptimer_alarm_config_t alarm_config = {
		.reload_count = 0,
		.alarm_count = 20000,
		.flags.auto_reload_on_alarm = true,
	};
	gptimer_event_callbacks_t cbs = { .on_alarm = pwm_callback };
	ESP_ERROR_CHECK(gptimer_new_timer(&timer_config, &pwm_irq_timer));
	ESP_ERROR_CHECK(gptimer_set_alarm_action(pwm_irq_timer, &alarm_config));
	ESP_ERROR_CHECK(gptimer_register_event_callbacks(pwm_irq_timer, &cbs, NULL));
	ESP_ERROR_CHECK(gptimer_enable(pwm_irq_timer));
	ESP_ERROR_CHECK(gptimer_start(pwm_irq_timer));
}

void done_pb_output_info() {
	gptimer_del_timer(pwm_irq_timer);
	free(pwms);
}

void output_set_power(int channel, char enable) {
	atomic_store(&(pwms[channel].off), enable);
	update_pwm(channel);
}

void output_toggle(int channel) {
	atomic_fetch_xor(&(pwms[channel].off), 1);
	update_pwm(channel);
}

void output_set_value(int channel, int level) {
	atomic_store(&(pwms[channel].output), level);
	pwms[channel].cyclesLeft = 0;
	update_pwm(channel);
}

// cs = 1 means 100 cs = 0.1s on
void output_set_value_timeout(int channel, int level, int cs) {
	if(atomic_load(&pwms[channel].off))
		return;
	uint16_t k = atomic_exchange(&pwms[channel].output, level);
	atomic_store(&pwms[channel].old_output, k);
	
	ledc_set_duty_and_update(LEDC_LOW_SPEED_MODE, channel, level, 0);
	pwms[channel].cyclesLeft = cs*5;
	pwms[channel].was_updated = 1;
}

int output_get_value(int channel) {
	return atomic_load(&(pwms[channel].output));
}

char is_off(int channel) {
	return atomic_load(&(pwms[channel].off));
}

void pwm_timeout_add_value(int channel, int increment) {
	uint16_t k = atomic_load(&(pwms[channel].output));
	if(0xffff - k < increment) {
		k = 0xffff;
	} else if(k < -increment) {
		k = 0;
	} else {
		k += increment;
	}
	atomic_store(&(pwms[channel].output), k);
}

void output_add_value(int channel, int increment) {
	if(increment > 0)
		atomic_fetch_add(&(pwms[channel].output), increment);
	else if(atomic_load(&(pwms[channel].output)) >= -increment)
		atomic_fetch_sub(&(pwms[channel].output), -increment);
	if(atomic_load(&(pwms[channel].output)) > 16383)
		atomic_store(&(pwms[channel].output), 16383);
	pwms[channel].cyclesLeft = 0;
	update_pwm(channel);
}

char output_was_updated(int channel) {
	char x = pwms[channel].was_updated;
	pwms[channel].was_updated = 0;
	return x;
}
