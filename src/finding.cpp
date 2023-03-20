#include "finding.hpp"

// TODO 
string linkify(path link_path);

bool Finding::wasFound() { return _found; }

string Finding::getFinding() { return _finding; }

Finding::Finding(path finding) : _finding(finding) {
    if (finding == "") _found = false;
    else _found = true;
}

