#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "../c/corsa_ffi/include/corsa_utils.h"
#include "corsa_utils.hpp"

namespace corsa::api {

inline std::string take_last_error() {
  return utils::take_string(corsa_error_message_take());
}

class tsgo_api_client {
 public:
  tsgo_api_client() = default;
  explicit tsgo_api_client(CorsaTsgoApiClient *handle) : handle_(handle) {}

  tsgo_api_client(const tsgo_api_client &) = delete;
  tsgo_api_client &operator=(const tsgo_api_client &) = delete;

  tsgo_api_client(tsgo_api_client &&other) noexcept : handle_(std::exchange(other.handle_, nullptr)) {}
  tsgo_api_client &operator=(tsgo_api_client &&other) noexcept {
    if (this != &other) {
      reset();
      handle_ = std::exchange(other.handle_, nullptr);
    }
    return *this;
  }

  ~tsgo_api_client() { reset(); }

  static tsgo_api_client spawn(std::string_view options_json) {
    return tsgo_api_client(corsa_tsgo_api_client_spawn(utils::to_ref(options_json)));
  }

  explicit operator bool() const { return handle_ != nullptr; }

  std::string initialize_json() const {
    return utils::take_string(corsa_tsgo_api_client_initialize_json(handle_));
  }

  std::string parse_config_file_json(std::string_view file) const {
    return utils::take_string(corsa_tsgo_api_client_parse_config_file_json(handle_, utils::to_ref(file)));
  }

  std::string update_snapshot_json(std::string_view params_json = {}) const {
    return utils::take_string(corsa_tsgo_api_client_update_snapshot_json(handle_, utils::to_ref(params_json)));
  }

  std::optional<std::vector<std::uint8_t>> get_source_file(
      std::string_view snapshot,
      std::string_view project,
      std::string_view file) const {
    return utils::take_bytes(corsa_tsgo_api_client_get_source_file(
        handle_,
        utils::to_ref(snapshot),
        utils::to_ref(project),
        utils::to_ref(file)));
  }

  std::string get_string_type_json(std::string_view snapshot, std::string_view project) const {
    return utils::take_string(corsa_tsgo_api_client_get_string_type_json(
        handle_, utils::to_ref(snapshot), utils::to_ref(project)));
  }

  std::string get_type_at_position_json(
      std::string_view snapshot,
      std::string_view project,
      std::string_view file,
      std::uint32_t position) const {
    return utils::take_string(corsa_tsgo_api_client_get_type_at_position_json(
        handle_,
        utils::to_ref(snapshot),
        utils::to_ref(project),
        utils::to_ref(file),
        position));
  }

  std::string get_symbol_at_position_json(
      std::string_view snapshot,
      std::string_view project,
      std::string_view file,
      std::uint32_t position) const {
    return utils::take_string(corsa_tsgo_api_client_get_symbol_at_position_json(
        handle_,
        utils::to_ref(snapshot),
        utils::to_ref(project),
        utils::to_ref(file),
        position));
  }

  std::string type_to_string(
      std::string_view snapshot,
      std::string_view project,
      std::string_view type_handle,
      std::string_view location = {},
      std::int32_t flags = -1) const {
    return utils::take_string(corsa_tsgo_api_client_type_to_string(
        handle_,
        utils::to_ref(snapshot),
        utils::to_ref(project),
        utils::to_ref(type_handle),
        utils::to_ref(location),
        flags));
  }

  std::string call_json(std::string_view method, std::string_view params_json = {}) const {
    return utils::take_string(corsa_tsgo_api_client_call_json(
        handle_, utils::to_ref(method), utils::to_ref(params_json)));
  }

  std::optional<std::vector<std::uint8_t>> call_binary(
      std::string_view method,
      std::string_view params_json = {}) const {
    return utils::take_bytes(corsa_tsgo_api_client_call_binary(
        handle_, utils::to_ref(method), utils::to_ref(params_json)));
  }

  bool release_handle(std::string_view handle) const {
    return corsa_tsgo_api_client_release_handle(handle_, utils::to_ref(handle));
  }

  bool close() {
    if (handle_ == nullptr) {
      return true;
    }
    auto *handle = std::exchange(handle_, nullptr);
    const bool ok = corsa_tsgo_api_client_close(handle);
    corsa_tsgo_api_client_free(handle);
    return ok;
  }

  void reset() {
    if (handle_ != nullptr) {
      corsa_tsgo_api_client_free(handle_);
      handle_ = nullptr;
    }
  }

 private:
  CorsaTsgoApiClient *handle_ = nullptr;
};

}  // namespace corsa::api
