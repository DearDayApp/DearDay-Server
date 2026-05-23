# 이슈 트래커: GitHub

이 저장소의 이슈와 PRD는 GitHub 이슈로 관리한다. 모든 작업은 `gh` CLI로 수행한다.

## 컨벤션

- **이슈 생성**: `gh issue create --title "..." --body "..."`. 여러 줄 본문은 heredoc을 쓴다.
- **이슈 조회**: `gh issue view <number> --comments`. 코멘트는 `jq`로 필터링하고 라벨도 함께 가져온다.
- **이슈 목록**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`. 필요하면 `--label`, `--state` 필터를 함께 쓴다.
- **이슈에 댓글**: `gh issue comment <number> --body "..."`
- **라벨 추가/제거**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **이슈 종료**: `gh issue close <number> --comment "..."`

저장소는 `git remote -v`에서 추론한다 — clone 안에서 `gh`를 실행하면 자동으로 인식한다.

## 스킬이 "이슈 트래커에 게시"하라고 할 때

GitHub 이슈를 생성한다.

## 스킬이 "관련 티켓을 가져오라"고 할 때

`gh issue view <number> --comments`를 실행한다.
