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
using std::filesystem::path;

string Link::hugo_link() { return "[" + _name + "](" + _link.string() + ")"; }

path relative_to(path file_path, path relative) {
    string path_string = file_path.string();
    string other_string = relative.string();
    
    if (other_string == path_string.substr(0, other_string.length())) {
        return path_string.substr(other_string.length() + 1);
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

Finding find_file(path dir, string name) {
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

path find_file_in_vault(path vault, string name) {
    Finding finding = find_file(vault, name);

    if (finding.was_found()) {
        return relative_to(finding.get_finding(), vault);
    } else {
        // TODO print warning
        cout << "WARNING: coud not find link \'" << name << "\'" << endl; 
        return vault;
    }
}


void Converter::_convert_dir(path dir, path out_dir) {
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        string ext = file.path().extension();
        path out_path = out_dir.string() + "/" + relative_to(file.path(), _vault).string();

        std::filesystem::create_directory(out_dir);

        if (file.is_directory()) {
            std::filesystem::create_directory(out_path);
            _convert_dir(file.path(), out_dir);
        } else if (ext == ".md") {
            string file_contents = read_file(file.path());
            string hugo_contents = _obsidian_to_hugo(file.path(), file_contents);
            write_file(out_path, hugo_contents);
            cout << "Wrote file: " << out_path << endl;
        }
    }
}

string _add_header(path file_path, string contents){

    string header = "---\ntitle: " + file_path.stem().string() + "\n---";

    return header + contents;
}

string Converter::_obsidian_to_hugo(path file_path, string content) {
    content = _hugoify_links(file_path, content);

    content = _add_header(file_path, content);
    return content;
}

int Converter::dir_debth(path path){
    string path_string = path.string();
    string vault_string = _vault.string();
    size_t n = std::count(path_string.begin(), path_string.end(), '/') - std::count(vault_string.begin(), vault_string.end(), '/') - 1;
    return n;
}

string Converter::_hugoify_links(path file_path, string content) {
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

void Converter::convert_vault(path out_dir) {
    _convert_dir(_vault, out_dir);
}

int main(int argc, char *argv[]) {
	string vault_path = "/home/balder/Documents/uni/noter";
	string out_dir = "/home/balder/projects/website/content/notes";

    Converter c = Converter(vault_path);
    c.convert_vault(out_dir);

	return 0;
}
