#include <vector>
#include <iostream>
#include <filesystem>

using std::string;
using std::filesystem::path;

class Link;
class Finding;

Finding find_file(string dir, string name);
string read_file(std::string path);
Finding find_file(path dir, string name);
path find_file_in_vault(path vault, string name);
path relative_to(path file_path, path relative);


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
        static Link link_from_raw(path vault, string full_input){

            // -> link#chapter|name
            path link;
            string chapter;
            string name;

            int bar_index = full_input.find("|");
            int hash_index = full_input.find("#");
            bool bar = bar_index != -1;
            bool hash = hash_index != -1;

            if (!bar and !hash){
                link = find_file_in_vault(vault, full_input);
                name = full_input;
                chapter = "";
            }
            else if (bar and !hash) {
                link = find_file_in_vault(vault, full_input.substr(0, bar_index));
                name = full_input.substr(bar_index + 1);
                chapter = "";
            }
            else if (!bar and hash) {
                link = find_file_in_vault(vault, full_input.substr(0, hash_index));
                chapter = full_input.substr(hash_index + 1);
                name = full_input;
            }
            else if (bar and hash) {
                link = find_file_in_vault(vault, full_input.substr(0, hash_index));
                chapter = full_input.substr(hash_index + 1, bar_index);
                name = full_input.substr(bar_index + 1);
            }
            return Link(name, link, chapter);
        }

        string hugo_link(path vault, path hugo_path);
        bool has_destination();
};


class Converter {
    path _vault;

	void _convert_dir(path dir, path out_dir, path hugo_path);
	string _hugoify_links(path file_path, path hugo_path, string content);
	string _obsidian_to_hugo(path file_path, path hugo_path, string content);
    int dir_debth(path path);
    string _double_newlines(string content);
    string _add_header(path file_path, string contents);

public:
    void convert_vault(path out_dir, path hugo_path);
    Converter(path vault) : _vault(vault) {}
};
