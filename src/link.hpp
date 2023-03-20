#ifndef LINK_INCLUDES
#define LINK_INCLUDES

#include <filesystem>

#include "converter.hpp"
#include "note.hpp"

using std::string;
using std::filesystem::path;

class Link {
    path _vault_path;
    string _name;
    string _chapter;
    bool _shown;

    string _chapterId();

    public:
        Link(string name, path link, string chapter = "", bool shown = false);
        static Link linkFromRaw(path vault, string full_input, Converter* converter);
        path hugoLink(path hugo_path);
        string markdownLink(path hugo_vault_path);
        string newTabLink(path hugo_vault_path);
        void doNotShow();
        bool hasDestination();
        path getVaultPath();
        string getName();
};

#endif // !LINK_INCLUDES
