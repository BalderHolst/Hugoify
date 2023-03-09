#include <filesystem>

using std::string;
using std::filesystem::path;


class Finding {
    bool _found;
    path _finding;

    public:
        Finding(path finding = "") : _finding(finding) {
            if (finding == "") _found = false;
            else _found = true;
        }
        bool was_found();
        string get_finding();
};
