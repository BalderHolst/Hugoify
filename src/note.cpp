#include "note.hpp"

#include <filesystem>
#include <vector>

using std::filesystem::path;
using std::vector;

Note::Note(path vault, path obsidian_path, path hugo_dst) : _obsidian_path(obsidian_path) {
        _hugo_path = hugo_dst / obsidian_path.lexically_relative(vault);
}

path Note::getObsidianVaultPath() {
    return _obsidian_path;
}

path Note::getHugoPath() {
    return _hugo_path;
}

path Note::getWebPath(path hugo_dst) {
    return _hugo_path.lexically_relative(hugo_dst);
}

void Note::addBacklink(Note* note_ref){
    _backlinks.push_back(note_ref);
}

vector<Note*> Note::getBacklinks(){
    return _backlinks;
}

