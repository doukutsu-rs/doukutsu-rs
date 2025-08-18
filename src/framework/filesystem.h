#pragma once

#include "../common.h"
#include <memory>
#include <vector>
#include <string>
#include <fstream>

namespace doukutsu {
namespace framework {

/// Virtual file system abstraction
class Filesystem {
public:
    Filesystem();
    ~Filesystem();

    /// Open a file for reading
    [[nodiscard]] std::unique_ptr<std::istream> open(const std::string& path);
    
    /// Check if a file exists
    [[nodiscard]] bool exists(const std::string& path) const;
    
    /// Get the full path to a file
    [[nodiscard]] std::string get_absolute_path(const std::string& path) const;

private:
    std::vector<std::string> search_paths_;
};

} // namespace framework
} // namespace doukutsu