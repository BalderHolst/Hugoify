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
        switch (s[i]) {
            case ' ': s[i] = '-'; break;
            case '(':
            case ')': s = s.substr(0, i) + s.substr(i + 1); break;
        }
    }
    return s;
}

void Note::addBacklink(Note* note_ref){
    note_ref->_backlinks.push_back(this);
}

vector<Note*> Note::getBacklinks(){
    return _backlinks;
}

