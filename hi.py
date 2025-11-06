import tkinter as tk
from tkinter import messagebox


def show_win():
    messagebox.showinfo("Результат", "победа")


def show_lose():
    messagebox.showinfo("Результат", "проигрыш")


root = tk.Tk()
root.title("Две кнопки")

win_btn = tk.Button(root, text="Показать: победа", command=show_win, width=20)
lose_btn = tk.Button(root, text="Показать: проигрыш", command=show_lose, width=20)

win_btn.pack(padx=20, pady=10)
lose_btn.pack(padx=20, pady=10)

root.mainloop()


