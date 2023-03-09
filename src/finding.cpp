#include "finding.hpp"

// TODO 
string linkify(path link_path);

bool Finding::was_found() { return _found; }

string Finding::get_finding() { return _finding; }

Finding::Finding(path finding) : _finding(finding) {
    if (finding == "") _found = false;
    else _found = true;
}

