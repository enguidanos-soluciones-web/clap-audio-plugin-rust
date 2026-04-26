#pragma once

#include "activations.h"
#include "dsp.h"
#include "rust/cxx.h"
#include <cstdint>
#include <memory>

inline void activation_enable_fast_tanh() {
  nam::activations::Activation::enable_fast_tanh();
}

struct NamDsp {
  std::unique_ptr<nam::DSP> inner;
};

std::unique_ptr<NamDsp> dsp_load(rust::Str json);

void dsp_process(NamDsp &dsp, rust::Slice<const double> input,
                 rust::Slice<double> output);

void dsp_reset(NamDsp &dsp, double sample_rate, int32_t max_block_size);

double get_sample_rate_from_nam_file(rust::Str json);
