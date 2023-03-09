#include "converter.hpp"
#include "link.hpp"
#include "note.hpp"

#include "glob/single_include/glob/glob.hpp"

#include <filesystem>
#include <iostream>
#include <regex>
#include <vector>

using std::cout;
using std::endl;
using std::vector;

namespace fs = std::filesystem;

// TODO do something about this
string read_file(std::string path);
void write_file(string path, string contents);
string linkify(path link_path);

Converter::Converter(path vault) : _vault(vault) {
    _excluded_paths = _get_excluded();
    _notes = _findNotes(vault);
}

bool Converter::_is_excluded(path file_path){
    for (path excluded_path : _excluded_paths) {
        if (file_path == excluded_path) return true;
    }
    return false;
}

void Converter::_convert_dir(path dir, path out_dir, path hugo_path) {
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        string ext = file.path().extension();
        path out_path = out_dir.string() + "/" + (file.path().lexically_relative(_vault)).string();

        std::filesystem::create_directory(out_dir);

        if (_is_excluded(file.path())) {
            cout << "Excluding " << file.path() << endl;
            continue;
        }

        if (file.is_directory()) {
            std::filesystem::create_directory(out_path);
            _convert_dir(file.path(), out_dir, hugo_path);
        } else if (ext == ".md") {
            string file_contents = read_file(file.path());
            string hugo_contents = _obsidian_to_hugo(file.path(), hugo_path, file_contents);
            write_file(out_path, hugo_contents);
            /* cout << "Wrote file: " << out_path << endl; */
        }
    }
}

string Converter::_add_header(path file_path, string contents){

    string header = "---\ntitle: " + file_path.stem().string() + "\n---";

    return header + contents;
}

// TODO optimise...
string Converter::_double_newlines(string content){
    for (int i = content.length(); i >= 0; i--) {
        if (content[i] == '\n'){
            content = content.substr(0, i) + "\n" + content.substr(i);
            i--;
        }
    }

    return content;
}

string Converter::_obsidian_to_hugo(path file_path, path hugo_path, string content) {
    cout << "Scraping content from: " << file_path.string() << endl;
    content = _hugoify_links(file_path, hugo_path, content);
    content = _double_newlines(content);

    content = _add_header(file_path, content);
    return content;
}

string Converter::_hugoify_links(path file_path, path hugo_path, string content) {
    std::smatch m;
    std::regex r("\\[\\[([^\\[\\]]+)\\]\\]");
    while (std::regex_search(content, m, r)) {
        string ms = m[1].str();
        Link link = Link::link_from_raw(_vault, ms, this);

        content = content.substr(0, m.position()) + link.hugo_markdown_link(_vault, hugo_path) +
                            content.substr(m.position() + m.length());
    }
    return content;
}

void Converter::convert_vault(path out_dir, path hugo_path) {
    fs::remove_all(out_dir);
    _convert_dir(_vault, out_dir, hugo_path);
}

std::vector<path> Converter::_get_excluded() {

    path exclude_path = _vault.string() + "/.export-ignore";

    if (!std::filesystem::exists(exclude_path)) return {};

    string exclude_string = read_file(exclude_path);

    std::vector<string> globs = {};

    int newline_location;
    string glob;
    do {
        newline_location = exclude_string.find("\n");
        glob = exclude_string.substr(0, newline_location);
        exclude_string = exclude_string.substr(newline_location + 1);
        if (glob != "") globs.push_back(glob);
    } while (newline_location != -1);

    std::vector<path> paths = {};

    for (string g : globs) {
        for (path& p : glob::glob(_vault.string() + "/" + g)) {
            string p_string = p.string();
            if (p_string[p_string.length() - 1] == '/') p = ((path) p_string.substr(0, p_string.length() - 1));
            paths.push_back(p);
        }
    }

    cout << "Excluded paths:" << endl;
    for (path p : paths) {
        cout << "\t" << p << endl;
    }
    cout << endl;

    return paths;
}

vector<Note> Converter::_findNotes(path dir){
    vector<Note> notes = {};

    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        path file_path = file.path();
        if (file.is_directory()) {
            _findNotes(dir);
        } else if (file_path.extension() == ".md") {
            notes.push_back(Note(_vault, file_path));
        }
    }

    return notes;
}

Finding Converter::_find_file(path dir, string name) {
    name = linkify(name);
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        if (file.is_directory()) {
            Finding finding = _find_file(file.path(), name);
            if (finding.was_found())
                return finding;
        } else {
            string filename = linkify(file.path().filename());

            if (filename[0] == '/') filename = filename.substr(1);

            if (filename == name) {
                return Finding(file.path());
            }
        }
    }
    return Finding("");
}


path Converter::find_file(string name) {

    name = ((path) name).filename().string(); // Isolate filename

    Finding finding = _find_file(_vault, name);

    if (finding.was_found()) {
        return ((path) finding.get_finding()).lexically_relative(_vault);
    } else {
        cout << "WARNING: coud not find link \'" << name << "\'" << endl; 

        return ""; // TODO Make this not create a link.
    }
}
