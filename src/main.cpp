#include <algorithm>
#include <filesystem>
#include <iostream>
#include <string>
#include <vector>
#include <fstream>
#include <regex>

using std::string;
using std::cout;
using std::endl;


string find_file(string dir, string name);
string find_file_in_vault(string vault, string name);

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

        string hugo_link(){
            return "[" + _name + "](" + _link + ")";
        }
};

string relative_to(string path, string relative) {
	if (relative == path.substr(0, relative.length())) {
		return path.substr(relative.length() + 1);
	}
	exit(1); // TODO
}

string find_file_in_vault(string vault, string name){
    string absolute = find_file(vault, name);
    return absolute.substr(vault.length() + 1);
}

// TODO
string find_file(string dir, string name){
	for (const auto &file : std::filesystem::directory_iterator(dir)) {
        cout << file.path().filename() << " == " <<  name + ".md" << endl;
		if (file.is_directory()) {
            cout << "found dir: " << file.path() << endl;
            find_file(file.path(), name);
        }
		 else if (file.path().filename() == name or file.path().filename() == name + ".md") {
            cout << "found file: " << file.path() << endl;
             return file.path();
		}
	}
    return dir + "/[NOT FOUND]";
}


string hugoify_links(string vault, string content){

    std::vector<Link> links = {};

    string s = content; // copy
    std::smatch m;
    std::regex r ("\\[\\[([^\\[\\]]+)\\]\\]" );

    while(std::regex_search(s, m, r)){

        string ms = m[1].str();

        links.push_back(Link::link_from_raw(vault, ms));
        s = m.suffix(); // remove content until match and match again
    }

    for (Link l : links) {
        cout << l.hugo_link() << endl;
    }

    return content;
}

string obsidian_to_hugo(string vault, string content){
    content = hugoify_links(vault, content);
    return content;
}

// Copy pasted code... 
string read_file(std::string path) {
    constexpr auto read_size = std::size_t(4096);
    auto stream = std::ifstream(path.data());
    stream.exceptions(std::ios_base::badbit);
    
    auto out = std::string();
    auto buf = std::string(read_size, '\0');
    while (stream.read(& buf[0], read_size)) {
        out.append(buf, 0, stream.gcount());
    }
    out.append(buf, 0, stream.gcount());
    return out;
}


void convert_dir(string dir, string out_dir, string vault_root = "") {
	if (vault_root == "")
		vault_root = dir;

	for (const auto &file : std::filesystem::directory_iterator(dir)) {
		string ext = file.path().extension();

		if (file.is_directory()) {
			// Create out dir here
			convert_dir(file.path(), out_dir, vault_root);
		} else if (ext == ".md") {
			string out_file = out_dir + "/" + relative_to(file.path(), vault_root);

			/* std::ifstream myfile; myfile.open(file.path()); */
			std::cout << read_file(file.path()) << std::endl;

			std::cout << "Found note: " << out_file << std::endl;
		}
	}
}

int main(int argc, char *argv[]) {
	string vault_path = "/home/balder/Documents/uni/noter";
	string out_dir = "./out";

	/* convert_dir(vault_path, out_dir); */

    obsidian_to_hugo(vault_path, read_file("/home/balder/Documents/uni/noter/notes/Skrå Kast.md"));

	return 0;
}
