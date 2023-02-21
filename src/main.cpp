#include <algorithm>
#include <filesystem>
#include <iostream>
#include <string>
#include <vector>
#include <fstream>
#include <regex>
#include "main.hpp"

using std::string;
using std::cout;
using std::endl;

string Link::hugo_link(){
    return "[" + _name + "](" + _link + ")";
}

string relative_to(string path, string relative) {
	if (relative == path.substr(0, relative.length())) {
		return path.substr(relative.length() + 1);
	}
	exit(1); // TODO
}

bool Finding::was_found(){
    return _found;
}
string Finding::get_finding(){
    return _finding;
}

string find_file_in_vault(string vault, string name){
    Finding finding = find_file(vault, name);

    if (finding.was_found()) {
        return finding.get_finding().substr(vault.length() + 1);
    }
    else {
        // TODO print warning
        return vault + "[NOT FOUND]";
    }
}

Finding find_file(string dir, string name){
	for (const auto &file : std::filesystem::directory_iterator(dir)) {
		if (file.is_directory()) {
            Finding finding = find_file(file.path(), name);
            if (finding.was_found()) return finding;
        }
		 else { 
                if ((string) file.path().filename() == name or (string) file.path().filename() == name + ".md") {
                    return Finding(file.path());
            }
		}
	}
    return Finding("");
}


string hugoify_links(string vault, string content){
    std::smatch m;
    std::regex r ("\\[\\[([^\\[\\]]+)\\]\\]" );

    while(std::regex_search(content, m, r)){

        string ms = m[1].str();

        Link link = Link::link_from_raw(vault, ms);
        content = content.substr(0, m.position()) + link.hugo_link() + content.substr(m.position() + m.length());
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
			/* std::cout << read_file(file.path()) << std::endl; */

			std::cout << "Found note: " << out_file << std::endl;
		}
	}
}

int main(int argc, char *argv[]) {
	string vault_path = "/home/balder/Documents/uni/noter";
	string out_dir = "./out";

	/* convert_dir(vault_path, out_dir); */

    cout << obsidian_to_hugo(vault_path, read_file("/home/balder/Documents/uni/noter/notes/Skrå Kast.md")) << endl;

	return 0;
}
