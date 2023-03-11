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
void write_file(path file_path, string contents);
string linkify(path link_path);

Converter::Converter(path vault, path hugo_root, path content_dir) : _vault(vault), _hugo_root(hugo_root), _content_dir(content_dir) {
    _excluded_paths = _get_excluded();
    _notes = _findNotes(vault);
}

void Converter::convert_vault() {
    fs::remove_all(_hugo_root / "content" / _content_dir);
    fs::remove_all(_hugo_root / "static" / _content_dir);

    for (int i = 0; i < _notes.size(); i++){
        Note* note = &_notes[i];
        path obsidian_path = _vault / note->getVaultPath();
        string hugo_contents = _obsidian_to_hugo(note);
        write_file(_hugo_root / "content" / _content_dir / note->getHugoVaultPath(), hugo_contents);
    }

    cout << "\nAdding backlinks..." << endl;
    //Add backlinks
    for (int i = 0; i < _notes.size(); i++){
        _addBacklinks(&_notes[i]);
    }
}

void Converter::_addBacklinks(Note* note){
    cout << note->getVaultPath() << endl;

    string yaml_backlinks = "backlinks: [";

    auto backlinks = note->getBacklinks();
    if (backlinks.size() > 0) {
        for (Note* backlink : backlinks) {
            if (_is_excluded(_vault / backlink->getVaultPath())) continue;
            cout << "\t" << backlink->getVaultPath() << endl;
            string yaml_backlink = linkify(backlink->getVaultPath().string());
            yaml_backlinks += "/" + yaml_backlink + ", ";
        }
        yaml_backlinks = yaml_backlinks.substr(0, yaml_backlinks.length()-2) + "]";
    }
    else {
        yaml_backlinks += "]";
    }

    path hugo_file_path = _hugo_root / "content" / _content_dir / note->getHugoVaultPath();
    string content = read_file(hugo_file_path);

    int first_return = content.find("\n");

    content = content.substr(0, first_return) +
        "\n" + yaml_backlinks +
        content.substr(first_return);

    write_file(hugo_file_path, content);
}

bool Converter::_is_excluded(path file_path){
    for (path excluded_path : _excluded_paths) {
        if (file_path == excluded_path) return true;
    }
    return false;
}

string Converter::_add_header(path file_path, string contents){
    string header = "---\ntype: note\ntitle: " + file_path.stem().string() + "\n---\n\n";
    return header + contents;
} 

string Converter::_double_newlines(string content){
    bool codeblock = false;
    for (int pos = content.find('\n'); pos != -1; pos = content.find('\n', pos + 1)) {
        if (pos > 0 && content[pos - 1] == '|') continue; // if table
        else if (codeblock) continue;
        else if (pos+3 < content.length()  && content[pos+1] == '`' && content[pos+2] == '`' && content[pos+3] == '`') {
            codeblock = !codeblock;
            continue;
        }
        else if (pos+1 < content.length() && content[pos+1] == '>') continue;
        content = content.substr(0, pos) + '\n' + content.substr(pos);
        pos++;
    }
    return content;
}

string Converter::_obsidian_to_hugo(Note* note) {
    path obsidian_path = _vault / note->getVaultPath();
    string content = read_file(obsidian_path);

    cout << "Scraping content from: " << obsidian_path.string() << endl;
    content = _hugoify_links(note, content);
    content = _double_newlines(content);

    content = _add_header(obsidian_path, content);
    return content;
}

Note* Converter::_getNote(path vault_path){
    /* cout << "Getting note: " << vault_path << endl; */
    for (int i = 0; i < _notes.size(); i++) {
        Note* note = &_notes[i];
        if (note->getVaultPath() == vault_path) {
            return note;
        }
    }
    cout << "WARNING: could not find note: " << vault_path << endl;
    return nullptr;
}

string Converter::_hugoify_links(Note* note, string content) {
    std::smatch m;
    std::regex r("(!?\\[\\[[^\\[\\]]+\\]\\])");
    while (std::regex_search(content, m, r)) {
        string ms = m[1].str();
        Link link = Link::link_from_raw(_vault, ms, this);

        path vault_path = link.getVaultPath();
        Note* bnote = _getNote(vault_path);

        string text_link;

        if (bnote != nullptr) {
            note->addBacklink(bnote);
            text_link = link.hugo_markdown_link(_content_dir);
        }
        else if (vault_path != "") {
            path hugo_path = _hugo_root / "static" / _content_dir / linkify(vault_path);
            fs::create_directories(hugo_path.parent_path());

            if (!fs::exists(hugo_path)) fs::copy_file(_vault / vault_path, hugo_path);

            text_link = link.new_tab_link(_content_dir);
        }
        else {
            std::cout << "WARNING: Removing link: " << link.getName() << std::endl;
            text_link = link.getName();
        }

        content = content.substr(0, m.position()) + text_link +
                            content.substr(m.position() + m.length());

    }
    return content;
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
        if (_is_excluded(file_path)) continue;
        else if (file.is_directory()) {
            for (Note note : _findNotes(file_path)) notes.push_back(note);
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
        cout << "WARNING: could not find link \'" << name << "\'" << endl; 

        return ""; // TODO Make this not create a link.
    }
}
