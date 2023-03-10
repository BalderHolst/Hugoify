#include "note.hpp"

#include <filesystem>
#include <vector>

#include <iostream>

using std::filesystem::path;
using std::vector;

Note::Note(path vault, path obsidian_path) {
    _vault_path = obsidian_path.lexically_relative(vault);
}

path Note::getVaultPath() {
    return _vault_path;
}

void Note::addBacklink(Note* note_ref){
    _backlinks.push_back(note_ref);
}

vector<Note*> Note::getBacklinks(){
    return _backlinks;
}

