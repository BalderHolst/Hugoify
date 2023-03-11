#include "note.hpp"
#include "finding.hpp"

#include <filesystem>
#include <vector>

#include <iostream>

using std::filesystem::path;
using std::vector;
using std::string;

Note::Note(path vault, path obsidian_path) {
    _vault_path = obsidian_path.lexically_relative(vault);
}

path Note::getVaultPath() {
    return _vault_path;
}

path Note::getHugoVaultPath() {
    string s = _vault_path;
    for (int i = 0; i < s.length(); i++) {
        if (s[i] == ' ') s[i] = '-';
    }
    return s;
}

void Note::addBacklink(Note* note_ref){
    _backlinks.push_back(note_ref);
}

vector<Note*> Note::getBacklinks(){
    return _backlinks;
}

