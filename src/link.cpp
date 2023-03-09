#include "link.hpp"

// TODO
path find_file_in_vault(path vault, string name);
string linkify(path link_path);

Link::Link(string name, path link, string chapter) : _name(name), _link(link), _chapter(chapter) {
    if (name == "") _name = link;
    _no_destination = link == "";
}

Link Link::link_from_raw(path vault, string full_input){

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
    else if (!bar and hash) { // TODO handle local links
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

path Link::hugo_link(path hugo_path){
    path p = hugo_path.string() + "/" + linkify(_link);
    return p;
}

string Link::hugo_markdown_link(path vault, path hugo_path) { 
    if (has_destination()) {
        string link = hugo_link(hugo_path);
        return "[" + _name + "](" + link + ")"; 
    }
    else {
        return _name;
    }
}

bool Link::has_destination(){
    return !_no_destination;
}

