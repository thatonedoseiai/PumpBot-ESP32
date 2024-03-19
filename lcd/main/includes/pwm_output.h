#ifndef PWM_OUTPUT_H
#define PWM_OUTPUT_H

#include "driver/ledc.h"
#include "stdatomic.h"
#include "driver/gptimer.h"

typedef struct pb_output_info_t_ {
    atomic_uint_fast16_t output;
    atomic_uchar off;
    uint16_t prev_value;
    atomic_uint_fast16_t cyclesLeft;
    atomic_uint_fast16_t old_output;
    char was_updated;
} pb_output_info_t;

void init_pb_output_info(void);
void done_pb_output_info(void);
void output_set_power(int channel, char enable);
void output_toggle(int channel);
void output_set_value(int channel, int level);
void output_set_value_timeout(int channel, int level, int ms);
int output_get_value(int channel);
char is_off(int channel);
void output_add_value(int channel, int increment);
char output_was_updated(int channel);

#endif
