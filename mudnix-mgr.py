#!/usr/bin/env python3
import sys
import json
from datetime import datetime


class UsersFile:
  def __init__(self, path):
    self._path = path
    self.data = None

  def load(self):
    file = open(self._path)
    self.data = json.loads(file.read())
    file.close()

  def save(self, of=None):
    if self.data is not None:
      out_path = self._path if of is None else of
      out_file = open(out_path, "w+")
      out_file.write(json.dumps(self.data, indent=2))
      out_file.close()


def print_user(user):
  creation_time = datetime.utcfromtimestamp(
    user['account_creation_timestamp']
  ).strftime('%Y-%m-%d %H:%M:%S UTC')
  active_time = datetime.utcfromtimestamp(
    user['last_activity_timestamp']
  ).strftime('%Y-%m-%d %H:%M:%S UTC')
  print(f"User: {user['username']}")
  print(f"* Account created:\t{creation_time}")
  print(f"* Last time active:\t{active_time}")
  print(f"* Current location:\t{user['world_location']}\n")


def print_current_time():
  print(
    "Current time: "
    + datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')
    + "\n"
  )


def main(argv):
  users_file = UsersFile("users.json")
  if argv[1] == "--list":
    print_current_time()
    users_file.load()
    for user in users_file.data["users"]:
      print_user(user)
  elif argv[1] == "--userdel":
    users_file.load()
    users_to_delete = argv[2:]
    users_file.data["users"] = [
      user for user in users_file.data["users"] if (
        user["username"] not in users_to_delete
      )
    ]
    users_file.save()
  elif argv[1] == "--userinfo":
    print_current_time()
    users_file.load()
    users_to_print = argv[2:]
    for user in filter(
      lambda u: u["username"] in users_to_print, users_file.data["users"]
    ):
      print_user(user)


if __name__ == "__main__":
  main(sys.argv)
