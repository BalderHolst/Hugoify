#include "converter.hpp"
#include "link.hpp"
#include "note.hpp"

#include "glob/single_include/glob/glob.hpp"

#include <cctype>
#include <filesystem>
#include <ios>
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

Converter::Converter(path vault, path hugo_root, path content_dir) : _vault(vault), _hugo_root(hugo_root) {
    _content_dir = content_dir / "vault";
    _excluded_paths = _getExcluded();
    _notes = _findNotes(vault);
}

void Converter::convertVault() {
    fs::remove_all(_hugo_root / "content" / _content_dir);
    fs::remove_all(_hugo_root / "static" / _content_dir);

    for (int i = 0; i < _notes.size(); i++){
        Note* note = &_notes[i];
        path obsidian_path = _vault / note->getVaultPath();
        string hugo_contents = _obsidianToHugo(note);
        write_file(_hugo_root / "content" / _content_dir / note->getHugoVaultPath(), hugo_contents);
    }

    cout << "\nAdding backlinks..." << endl;
    //Add backlinks
    for (int i = 0; i < _notes.size(); i++){
        _addBacklinks(&_notes[i]);
    }

    _addIndexFile();
}

void Converter::_addIndexFile(){
    string index = "---\n";
    index += "type: \"note\"\n";
    index += "---\n";
    write_file(_hugo_root / "content" / _content_dir / "_index.md", index);
}

void Converter::_addBacklinks(Note* note){
    cout << note->getVaultPath() << endl;

    string yaml_backlinks = "backlinks: [";

    auto backlinks = note->getBacklinks();
    if (backlinks.size() > 0) {
        for (Note* backlink : backlinks) {
            if (_isExcluded(_vault / backlink->getVaultPath())) continue;
            cout << "\t" << backlink->getVaultPath() << endl;
            string yaml_backlink = linkify(_content_dir / backlink->getVaultPath().string());
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

bool Converter::_isExcluded(path file_path){
    for (path excluded_path : _excluded_paths) {
        if (file_path.lexically_relative(excluded_path) == file_path.filename()) return true;
    }
    return false;
}

string Converter::_obsidianToHugo(Note* note) {
    path obsidian_path = _vault / note->getVaultPath();
    string content = read_file(obsidian_path);

    cout << "Scraping content from: " << obsidian_path.string() << endl;

    vector<string> tags = _extractTags(content);

    _hugoifyLinks(note, content);
    _formatCboxes(content);
    _formatLatex(content);

    _addHeader(obsidian_path, tags, content);
    return content;
}


void Converter::_formatCboxes(string& content){
    
    const string cbox_end = "{{< /cbox >}}\n\n";

    std::smatch m;
    std::regex r(">\\[!([^\\s\\n]+)\\](-)? ?(.*)\\n((>.*\\n)*)");
    while (std::regex_search(content, m, r)) {
        string type = m[1].str();
        string title = m[3].str();
        bool foldable = m[2] == "-";

        if (title == "") {
            title = type;
            title[0] = toupper(title[0]);
        }

        for (int i = 0; i < type.length(); i++) type[i] = tolower(type[i]);

        string cbox_begin = "{{< cbox type=\"" + type + "\" " +
            "title=\"" + title + "\" " +
            "foldable=\"" + (foldable ? "true" : "false") + "\" " +
            ">}}\n";
        
        string cbox_content = m[4].str();

        for (int i = 0; i != -1; i = cbox_content.find('>')) {
            cbox_content = cbox_content.substr(0, i) + cbox_content.substr(i + 1);
        }

        string cbox = cbox_begin + cbox_content + cbox_end;

        content = content.substr(0, m.position()) + cbox + content.substr(m.position() + m.length());
    }
}

vector<string> Converter::_extractTags(string& content){

    vector<string> tags = {};
    vector<int> hash_locations = {};
    bool in_codeblock = false;
    int content_ptr = 0;
    int hash_location;

    std::smatch m;
    std::regex r("(---\\n*)?(\\n|\\s)#([^\\s#]+)");
    while (std::regex_search(content, m, r)) {
        for (; content_ptr < m.position(); content_ptr++) {
            if ((content)[content_ptr] == '`') in_codeblock = !in_codeblock;
        }
        if (in_codeblock) {
            hash_location = content_ptr + m[1].length() + 1;
            (content)[hash_location] = ' '; // add replace tag to not match here anymore
            hash_locations.push_back(hash_location);
        }
        else{ 
            tags.push_back(m[3].str());
            content = content.substr(0, content_ptr) + content.substr(content_ptr + m.length());
        }
    }
    for (int l : hash_locations){ // Re-insert hashtags
        (content)[l] = '#';
    }
    return tags;
}

void Converter::_addHeader(path file_path, vector<string> tags, string& contents){
    string header = "---\ntype: note\ntitle: " + file_path.stem().string(); 

    if (tags.size() > 0){
        header += "\nnote-tags: [";
        for (string tag : tags) {
            header += tag + ", ";
        }
        header = header.substr(0, header.length()-2) + "]";
    }
    else {
        header += "\ntags: []";
    }
    header += "\n---\n\n";
    contents = header + contents;
} 

void Converter::_formatLatex(string& content){
    const string search_string = "$$";
    for (int pos = content.find(search_string); pos != -1; pos = content.find(search_string, pos + 2)) {
        int begin = pos;
        pos = content.find(search_string, pos + 2);
        int end = pos;

        for (int latex_pos = content.find("\\\\", begin); latex_pos != -1 && latex_pos < end; latex_pos = content.find("\\\\", latex_pos + 3)) {
            content = content.substr(0, latex_pos + 1) + "newline" + content.substr(latex_pos + 2);
        }
    }

    // Add needed newlines for inline math
    for (int latex_pos = content.find("\n$"); latex_pos != -1; latex_pos = content.find("\n$", latex_pos + 3)) {
        if (content[latex_pos + 2] == '$') continue;
        content = content.substr(0, latex_pos) + "\n" + content.substr(latex_pos);
        latex_pos++;
    }
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

void Converter::_hugoifyLinks(Note* note, string& content) {
    std::smatch m;
    std::regex r("(!?\\[\\[[^\\[\\]]+\\]\\])");
    while (std::regex_search(content, m, r)) {
        string ms = m[1].str();
        Link link = Link::linkFromRaw(_vault, ms, this);

        path vault_path = link.getVaultPath();
        Note* linked_note = _getNote(vault_path);

        string text_link;

        if (linked_note != nullptr) { // if the linked_note is a note
            note->addBacklink(linked_note);
            link.doNotShow();
            text_link = link.markdownLink(_content_dir);
        }
        else if (link.hasDestination()) { // if the file is an attachment
            text_link = link.markdownLink(_content_dir);

            // Copy attachment to website
            path hugo_path = _hugo_root / "static" / _content_dir / linkify(vault_path);
            fs::create_directories(hugo_path.parent_path());
            if (!fs::exists(hugo_path) && !_isExcluded(_vault / vault_path)) fs::copy_file(_vault / vault_path, hugo_path);
        }
        else {
            std::cout << "WARNING: Removing link: " << link.getName() << std::endl;
            text_link = link.getName();
        }

        content = content.substr(0, m.position()) + text_link +
                            content.substr(m.position() + m.length());

    }
}


std::vector<path> Converter::_getExcluded() {

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
        if (_isExcluded(file_path)) continue;
        else if (file.is_directory()) {
            for (Note note : _findNotes(file_path)) notes.push_back(note);
        } else if (file_path.extension() == ".md") {
            notes.push_back(Note(_vault, file_path));
        }
    }

    return notes;
}

Finding Converter::_findFile(path dir, string name) {
    name = linkify(name);
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        if (file.is_directory()) {
            Finding finding = _findFile(file.path(), name);
            if (finding.wasFound())
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


path Converter::findFile(string name) {

    name = ((path) name).filename().string(); // Isolate filename

    Finding finding = _findFile(_vault, name);

    if (finding.wasFound()) {
        return ((path) finding.getFinding()).lexically_relative(_vault);
    } else {
        cout << "WARNING: could not find link \'" << name << "\'" << endl; 

        return ""; // TODO Make this not create a link.
    }
}
