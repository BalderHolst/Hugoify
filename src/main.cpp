#include "main.hpp"
#include <algorithm>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <regex>
#include <string>
#include <vector>

using std::cout;
using std::endl;
using std::string;

string Link::hugo_link() { return "[" + _name + "](" + _link + ")"; }

string relative_to(string path, string relative) {
	if (relative == path.substr(0, relative.length())) {
		return path.substr(relative.length() + 1);
	}
	exit(1); // TODO
}

bool Finding::was_found() { return _found; }
string Finding::get_finding() { return _finding; }

// Copy pasted code...
string read_file(std::string path) {
    constexpr auto read_size = std::size_t(4096);
    auto stream = std::ifstream(path.data());
    stream.exceptions(std::ios_base::badbit);

    auto out = std::string();
    auto buf = std::string(read_size, '\0');
    while (stream.read(&buf[0], read_size)) {
        out.append(buf, 0, stream.gcount());
    }
    out.append(buf, 0, stream.gcount());
    return out;
}

void write_file(string path, string contents) {
    std::ofstream myfile;
    myfile.open(path);
    myfile << contents;
    myfile.close();
}

Finding find_file(string dir, string name) {
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        if (file.is_directory()) {
            Finding finding = find_file(file.path(), name);
            if (finding.was_found())
                return finding;
        } else {
            if (file.path().filename() == name or
                     file.path().filename() == name + ".md") {
                return Finding(file.path());
            }
        }
    }
    return Finding("");
}

string find_file_in_vault(string vault, string name) {
    Finding finding = find_file(vault, name);

    if (finding.was_found()) {
        return finding.get_finding().substr(vault.length() + 1);
    } else {
        // TODO print warning
        return vault + "[NOT FOUND]";
    }
}


void Converter::_convert_dir(string dir, string out_dir) {
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        string ext = file.path().extension();
        string out_path = out_dir + "/" + relative_to(file.path(), _vault);

        if (file.is_directory()) {
            std::filesystem::create_directory(out_path);
            _convert_dir(file.path(), out_dir);
        } else if (ext == ".md") {
            string file_contents = read_file(file.path());
            string hugo_contents = _obsidian_to_hugo(file.path(), file_contents);
            write_file(out_path, hugo_contents);
        }
    }
}

string Converter::_obsidian_to_hugo(string file_path, string content) {
    content = _hugoify_links(file_path, content);
    return content;
}

int Converter::dir_debth(string path){
    size_t n = std::count(path.begin(), path.end(), '/') - std::count(_vault.begin(), _vault.end(), '/') - 1;
    return n;
}

string Converter::_hugoify_links(string file_path, string content) {
    std::smatch m;
    std::regex r("\\[\\[([^\\[\\]]+)\\]\\]");
    while (std::regex_search(content, m, r)) {
        string ms = m[1].str();
        Link link = Link::link_from_raw(dir_debth(file_path), _vault, ms);
        content = content.substr(0, m.position()) + link.hugo_link() +
                            content.substr(m.position() + m.length());
    }
    return content;
}

void Converter::convert_vault(string out_dir) {
    _convert_dir(_vault, out_dir);
}

int main(int argc, char *argv[]) {
	string vault_path = "/home/balder/Documents/uni/noter";
	string out_dir = "./out";

    Converter c = Converter(vault_path);
    c.convert_vault(out_dir);

	return 0;
}
