#pragma once
#include <string>
#include <sstream>

std::string Display(int self) { return std::to_string(self); }
  
int Clone(int self) { return self; }
  
std::string Display(float self) { return std::to_string(self); }
  
float Clone(float self) { return self; }
  
std::string Display(const std::string& self) { return self; }
  
std::string Clone(const std::string& self) { return self; }
  
enum class Status : uint8
{
  Ok,
    Error,
    
};

std::string Display(Status self)
{
  std::stringstream result("Status::");
  switch self
  {
    case Status::Ok:
        result << "Ok";
        break;
      case Status::Error:
        result << "Error";
        break;
      
    default:
      result << "<UNKNOWN>";
      break;
  }
  return result;
}

Status Clone(Status self) {
  return self;
}

struct Foo
{
  int a;
    std::string b;
    Status c;
    float d;
    
};

std::string Display(const Foo & self)
{
  std::stringstream result("Foo\n{\n");
  result << "a: " << Display(self.a) << ",\n";
    result << "b: " << Display(self.b) << ",\n";
    result << "c: " << Display(self.c) << ",\n";
    result << "d: " << Display(self.d) << ",\n";
    
  result << "}";
  return result;
}

Foo Clone(const Foo & self) {
  Foo result = {
    Clone(self.a),
      Clone(self.b),
      Clone(self.c),
      Clone(self.d),
      
  };
  return result;
}

