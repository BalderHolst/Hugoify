#ifndef LINK_INCLUDES
#define LINK_INCLUDES

#include <filesystem>

#include "converter.hpp"
#include "note.hpp"

using std::string;
using std::filesystem::path;

class Link {
    bool _no_destination;
    path _vault_path;
    string _name;
    string _chapter;
    bool _shown;

    public:
        Link(string name, path link, string chapter = "", bool shown = false);
        static Link link_from_raw(path vault, string full_input, Converter* converter);
        path hugo_link(path hugo_path);
        string hugo_markdown_link(path hugo_vault_path);
        bool has_destination();
        path getVaultPath();
};

#endif // !LINK_INCLUDES
