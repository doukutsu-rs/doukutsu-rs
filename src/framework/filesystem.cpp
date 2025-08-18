#include "filesystem.h"
#include "error.h"
#include <filesystem>

namespace doukutsu {
namespace framework {

Filesystem::Filesystem() {
    // Add default search paths
    search_paths_.push_back("./");
    search_paths_.push_back("./data/");
    search_paths_.push_back("./builtin/");
}

Filesystem::~Filesystem() = default;

std::unique_ptr<std::istream> Filesystem::open(const std::string& path) {
    for (const auto& search_path : search_paths_) {
        std::string full_path = search_path + path;
        
        auto stream = std::make_unique<std::ifstream>(full_path, std::ios::binary);
        if (stream->is_open()) {
            return std::move(stream);
        }
    }
    
    throw GameError::io_error("Could not open file: " + path);
}

bool Filesystem::exists(const std::string& path) const {
    for (const auto& search_path : search_paths_) {
        std::string full_path = search_path + path;
        if (std::filesystem::exists(full_path)) {
            return true;
        }
    }
    return false;
}

std::string Filesystem::get_absolute_path(const std::string& path) const {
    for (const auto& search_path : search_paths_) {
        std::string full_path = search_path + path;
        if (std::filesystem::exists(full_path)) {
            return std::filesystem::absolute(full_path).string();
        }
    }
    return path; // Return original if not found
}

} // namespace framework
} // namespace doukutsu