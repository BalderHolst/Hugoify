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
        bool wasFound();
        string getFinding();
};

#endif // !FINDING_INCLUDES
