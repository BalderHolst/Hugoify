#ifndef LINK_INCLUDES
#define LINK_INCLUDES

#include <filesystem>

#include "converter.hpp"

using std::string;
using std::filesystem::path;

class Link {
    bool _no_destination;
    path _link;
    string _name;
    string _chapter;

    public:
        Link(string name, path link, string chapter = "");
        static Link link_from_raw(path vault, string full_input, Converter* converter);
        path hugo_link(path hugo_path);
        string hugo_markdown_link(path vault, path hugo_path);
        bool has_destination();
};

#endif // !LINK_INCLUDES
