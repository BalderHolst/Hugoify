#ifndef FINDING_INCLUDES
#define FINDING_INCLUDES

#include <filesystem>
#include <vector>

using std::string;
using std::filesystem::path;


class Finding {
    bool _found;
    path _finding;

    public:
        Finding(path finding = "");
        bool was_found();
        string get_finding();
};

#endif // !FINDING_INCLUDES
