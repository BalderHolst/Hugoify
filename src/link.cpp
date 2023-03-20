#include "link.hpp"
#include "converter.hpp"

#include <cctype>
#include <iostream>

// TODO
string linkify(path link_path);

Link::Link(string name, path vault_path, string chapter, bool shown) :
    _name(name),
    _vault_path(vault_path),
    _chapter(chapter),
    _shown(shown)
{
    if (name == "") _name = vault_path;
}

Link Link::link_from_raw(path vault, string full_input, Converter* converter){

    bool shown = false;

    // -> ![[link#chapter|name]]
    if (full_input[0] == '!'){
        shown = true;
        full_input = full_input.substr(1);
    }

    full_input = full_input.substr(2, full_input.length() - 4); // idk why it has to be 4 and not 2, but this works.

    path link;
    string chapter;
    string name;

    int bar_index = full_input.find("|");
    int hash_index = full_input.find("#");
    bool bar = bar_index != -1;
    bool hash = hash_index != -1;

    if (!bar and !hash){
        link = converter->find_file(full_input);
        name = full_input;
        chapter = "";
    }
    else if (bar and !hash) {
        link = converter->find_file(full_input.substr(0, bar_index));
        name = full_input.substr(bar_index + 1);
        chapter = "";
    }
    else if (!bar and hash) { // TODO handle local links
        link = converter->find_file(full_input.substr(0, hash_index));
        chapter = full_input.substr(hash_index + 1);
        name = full_input;
    }
    else if (bar and hash) {
        link = converter->find_file(full_input.substr(0, hash_index));
        chapter = full_input.substr(hash_index + 1, bar_index);
        name = full_input.substr(bar_index + 1);
    }
    return Link(name, link, chapter, shown);
}

string Link::_chapterId(){
    string c = _chapter;
    for (int i = c.length(); i >= 0; i--) {
        c[i] = std::tolower(c[i]);
        switch (c[i]) {
            case '.': c = c.substr(0, i) + c.substr(i + 1); break;
            case ' ': c[i] = '-';
        }
    }
    return c;
}

void Link::doNotShow(){
    _shown = false;
}

string Link::markdown_link(path hugo_vault_path) { 
    if (has_destination()) {
        string link = hugo_vault_path / linkify(_vault_path);
        if (_chapter != "") {
            link += "#" + _chapter;
        }

        string markdown_link = "[" + _name + "](/" + link + ")";

        if (_shown){
            markdown_link = "\n!" + markdown_link;
        }

        return markdown_link; 
    }
    else {
        return _name;
    }
}

string Link::new_tab_link(path hugo_vault_path){
    if (has_destination()) {
        string link = hugo_vault_path / linkify(_vault_path);
        return "{{< note_attachment name=\"" + _name + "\" link=\"/" + link  + "#" + _chapter + "\" >}}";
    }
    else {
        return _name;
    }
}

bool Link::has_destination(){
    return ( _vault_path != "" || _chapter != "");
}

path Link::getVaultPath(){
    return _vault_path;
}

string Link::getName() {
    return _name;
}
