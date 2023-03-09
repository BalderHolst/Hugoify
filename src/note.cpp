#include "note.hpp"

#include <filesystem>
#include <vector>

using std::filesystem::path;
using std::vector;

Note::Note(path vault, path obsidian_path) {
    _relative_path = obsidian_path.lexically_relative(vault);
}

path Note::getRelativePath() {
    return _relative_path;
}

void Note::addBacklink(Note* note_ref){
    _backlinks.push_back(note_ref);
}

vector<Note*> Note::getBacklinks(){
    return _backlinks;
}

