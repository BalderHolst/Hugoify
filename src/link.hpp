#include <filesystem>

using std::string;
using std::filesystem::path;

class Link {
    bool _no_destination;
    path _link;
    string _name;
    string _chapter;

    public:
        Link(string name, path link, string chapter = "") : _name(name), _link(link), _chapter(chapter) {
            if (name == "") _name = link;
            _no_destination = link == "";
        }
        static Link link_from_raw(path vault, string full_input);

        path hugo_link(path hugo_path);
        string hugo_markdown_link(path vault, path hugo_path);
        bool has_destination();
};
