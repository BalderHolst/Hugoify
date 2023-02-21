#include <vector>
#include <iostream>

using std::string;

class Link;
class Finding;

Finding find_file(string dir, string name);
string find_file_in_vault(string vault, string name);
string relative_to(string path, string relative);
string find_file_in_vault(string vault, string name);
Finding find_file(string dir, string name);
string hugoify_links(string vault, string content);
string obsidian_to_hugo(string vault, string content);
string read_file(std::string path);
void convert_dir(string dir, string out_dir, string vault_root);
int main(int argc, char *argv[]);

class Link {
    string _link;
    string _name;
    string _tag;

    public:
        Link(string name, string link, string tag = "") : _name(name), _link(link), _tag(tag) {
            if (name == "") _name = link;
        }
        static Link link_from_raw(string vault, string s){
            int link_sep = s.find("|");

            string link_name = link_sep != -1 ? s.substr(link_sep + 1) : "";

            string file_name_plus_tag = s.substr(0, link_sep);
            
            int tag_sep = s.find("#");

            string file_name = tag_sep != -1 ? file_name_plus_tag.substr(0, tag_sep) : file_name_plus_tag.substr(tag_sep + 1);

            string link = find_file_in_vault(vault, file_name);
            return Link(link_name, link);
        }
        string hugo_link();
};


class Finding {
    bool _found;
    string _finding;

    public:
        Finding(string finding = "") : _finding(finding) {
            if (finding == "") _found = false;
            else _found = true;
        }
        bool was_found();
        string get_finding();
};


