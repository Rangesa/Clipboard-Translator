from CodeVideoRenderer import CodeVideo

video = CodeVideo(
    code_string=open('eg.py').read(),   # ファイルから読み込む
    language='python',
    output='hello.mp4',                 # 出力ファイル名
    fps=30,                             # フレームレート
    theme='dracula',                    # カラーテーマ
    show_cursor=True,                   # カーソルのアニメーション
    line_duration=0.8                   # 行ごとの表示時間（秒）
)
video.render()
