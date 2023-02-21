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
    path _link;
    string _name;
    string _tag;


    public:
        Link(string name, path link, string tag = "") : _name(name), _link(link), _tag(tag) {
            if (name == "") _name = link;
        }
        static Link link_from_raw(int dir_debth, string vault, string s){
            int link_sep = s.find("|");


            string file_name_plus_tag = s.substr(0, link_sep);
            
            int tag_sep = s.find("#");

            string file_name = tag_sep != -1 ? file_name_plus_tag.substr(0, tag_sep) 
                : file_name_plus_tag;

            string link_name = link_sep != -1 ? s.substr(link_sep + 1) : file_name;

            string link = find_file_in_vault(vault, file_name);

            for (int i = 0; i < dir_debth; i++) {
                link = "../" + link;
            }

            return Link(link_name, link);
        }
        string hugo_link();
};


class Converter {
    path _vault;

	void _convert_dir(path dir, path out_dir);
	string _hugoify_links(path file_path, string content);
	string _obsidian_to_hugo(path file_path, string content);
    int dir_debth(path path);

public:
    void convert_vault(path out_dir);
    Converter(path vault) : _vault(vault) {}
};
