#pragma once

#include "dsp.h"
#include "rust/cxx.h"
#include <cstdint>
#include <memory>

struct NamDsp {
  std::unique_ptr<nam::DSP> inner;
};

std::unique_ptr<NamDsp> load(rust::Str json);

void process(NamDsp &dsp, rust::Slice<const double> input,
             rust::Slice<double> output);
void reset(NamDsp &dsp, double sample_rate, int32_t max_block_size);
void reset_and_prewarm(NamDsp &dsp, double sample_rate, int32_t max_block_size);
bool has_loudness(const NamDsp &dsp);
double get_loudness(const NamDsp &dsp);
double get_sample_rate_from_nam_file(rust::Str json);
int32_t check_nam_version_support(rust::Str json);
