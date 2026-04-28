#include "nam.h"
#include "get_dsp.h"
#include "json.hpp"
#include "model_config.h"
#include "wavenet/model.h"

namespace nam {
namespace lstm {
std::unique_ptr<ModelConfig> create_config(const nlohmann::json &, double);
}
namespace convnet {
std::unique_ptr<ModelConfig> create_config(const nlohmann::json &, double);
}
namespace linear {
std::unique_ptr<ModelConfig> create_config(const nlohmann::json &, double);
}
namespace container {
std::unique_ptr<ModelConfig> create_config(const nlohmann::json &, double);
}
} // namespace nam

static void ensure_registrations() {
  auto &reg = nam::ConfigParserRegistry::instance();

  if (!reg.has("WaveNet"))
    reg.registerParser("WaveNet", nam::wavenet::create_config);
  if (!reg.has("LSTM"))
    reg.registerParser("LSTM", nam::lstm::create_config);
  if (!reg.has("ConvNet"))
    reg.registerParser("ConvNet", nam::convnet::create_config);
  if (!reg.has("Linear"))
    reg.registerParser("Linear", nam::linear::create_config);
  if (!reg.has("SlimmableContainer"))
    reg.registerParser("SlimmableContainer", nam::container::create_config);
}

std::unique_ptr<NamDsp> dsp_load(rust::Str json) {
  ensure_registrations();
  auto config = nlohmann::json::parse(std::string(json.data(), json.size()));
  auto wrapper = std::make_unique<NamDsp>();
  wrapper->inner = nam::get_dsp(config);
  return wrapper;
}

void dsp_process(NamDsp &dsp, rust::Slice<const double> input,
                 rust::Slice<double> output) {
  double *in_ptr = const_cast<double *>(input.data());
  double *out_ptr = output.data();
  dsp.inner->process(&in_ptr, &out_ptr, static_cast<int>(input.size()));
}

void dsp_reset(NamDsp &dsp, double sample_rate, int32_t max_block_size) {
  dsp.inner->Reset(sample_rate, static_cast<int>(max_block_size));
}

double get_sample_rate_from_nam_file(rust::Str json) {
  auto config = nlohmann::json::parse(std::string(json.data(), json.size()));
  auto sampleRate = nam::get_sample_rate_from_nam_file(config);
  return sampleRate;
}
