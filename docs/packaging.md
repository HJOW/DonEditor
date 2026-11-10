# Don Editor 배포본 만들기

Don Editor는 GramEditor/Zed의 기존 기능을 유지하면서 로컬 MCP와 Agent 패널을 추가하는 GPL-3.0 계열 편집기다. 배포 스크립트는 GUI, CLI, MCP stdio 브릿지, 원저작·제3자·D2Coding 라이선스와 프로젝트 소스를 묶는다.

## Windows

저장소 루트에서 `package-zip.bat`를 실행한다. 누락된 빌드 도구는 설치 여부를 묻고, 거절하면 빌드 전에 중단한다. 실제 설치에는 WinGet 또는 Visual Studio Installer와 관리자 권한이 필요할 수 있다.

```powershell
.\package-zip.bat -CheckOnly
.\package-zip.bat
.\package-zip.bat -IncludeSource
```

결과는 `dist/DonEditor-<버전>-windows-<아키텍처>.zip`과 `.sha256`이다. 현재 제품 버전은 `0.1.1`이며 `Cargo.toml`의 워크스페이스 버전을 사용한다. 압축을 풀고 `doneditor.exe`를 실행한다. CLI는 `bin/doneditor.exe`, MCP 브릿지는 `bin/don-mcp.exe`다. `-BuildProfile Debug`는 개발 확인용이며 이름에 `-debug`를 붙인다. `-IncludeSource`는 기본 동봉 소스 zip을 `dist`에도 별도로 복사한다.

## Linux와 macOS

```sh
bash package-tar.sh --check-only
bash package-tar.sh
```

Linux/macOS 및 x64/ARM64 호스트를 감지한다. 도구가 부족하면 `install-build-tools.sh`의 번호 선택 메뉴를 연다. 필요한 항목을 선택하고 `I`로 설치를 확정한다. 취소·빈 입력·EOF·설치 실패 또는 설치 후 재검사 실패 시 빌드를 시작하지 않는다. Xcode 본체는 Apple 인증과 초기 실행을 사용자가 완료해야 하며, App Store를 연 것은 설치 완료로 취급하지 않는다.

`--dry-run`은 설치 메뉴에서 선택한 명령만 표시하며 빌드·압축은 수행하지 않는다. `--skip-check`는 도구가 준비된 CI/개발 환경용이다. `--skip-build`는 이미 생성된 `target/<호스트 타깃>/<release|debug>/` 산출물을 사용한다. `--debug`와 `--out-dir '결과물 경로'`도 지원한다. `CARGO_TARGET_DIR`를 설정했다면 해당 경로를 사용한다.

결과는 `dist/DonEditor-<버전>-<linux|macos>-<아키텍처>.tar.gz`와 `.sha256`이다. Linux에서는 압축 해제한 폴더의 `bin/doneditor`를 실행한다. macOS에서는 `Don Editor.app`을 열거나 `bin/doneditor`를 실행한다. 두 OS 모두 MCP 브릿지는 `bin/don-mcp`다. 원격 서버도 함께 빌드·포함한다. CLI의 기존 상대 경로 계약 때문에 일부 내부 파일명에 `gram`이 남아 있다.

Linux 배포본은 빌드한 배포판과 호환되는 glibc 및 GUI/시스템 동적 라이브러리를 요구한다. 시스템 라이브러리를 무조건 복사하는 독립 실행형 번들은 아니다. 다른 배포판에 제공하려면 해당 배포판에서 빌드하고 실행을 검증해야 한다. Linux/macOS 모두 Git 기능은 시스템에 설치된 Git을 사용한다. Agent API 키 저장에는 실행 중인 Secret Service와 D-Bus도 필요하다. macOS의 tar.gz는 개발용 비서명·비공증 `.app`을 포함한다. 정식 외부 배포의 Developer ID 서명과 Apple 공증은 배포자가 별도로 준비해야 한다.

## 대응 소스와 재배포

Windows zip은 `source/DonEditor-<버전>-source.zip`, Unix tar.gz는 `source/DonEditor-<버전>-source.tar.gz`를 **항상 동봉**한다. `git archive HEAD` 대신 패키징 시점의 작업 트리 파일을 읽어 미커밋 수정과 새 소스도 포함한다. Git이 추적하는 파일과 `.gitignore`에 해당하지 않는 새 파일을 포함하므로, 공개하면 안 되는 개인 파일은 먼저 `.gitignore`에 등록해야 한다. `target`, `dist`, 별도 출력 디렉터리와 `.git` 메타데이터는 제외한다. 삭제한 파일은 포함하지 않는다.

소스에는 프로젝트 코드, Cargo.lock, rust-toolchain.toml, 빌드 스크립트와 내장 자산이 들어 있다. 외부 Cargo 의존성은 잠금 파일에 지정한 레지스트리/저장소에서 받는다. `SOURCE.txt`에는 기준 커밋·빌드 대상·소스 위치와 원본 출처를 기록한다. 바이너리를 다시 배포할 때는 해당 소스와 `LICENSE`, `licenses.md`, `licenses/`를 함께 제공하고, 추가로 묶은 외부 라이브러리가 있다면 그 라이선스·소스 제공 조건도 적용해야 한다.

## 검증 범위

`tests/test_package_tar.py`는 실제 tar/gzip/git/SHA256을 사용하고 설치 도구·컴파일러는 가짜 명령으로 대체한다. Linux/macOS 분기, 명시적 설치·거절·EOF·재검사·실패, `.app` 메타데이터, 실행 권한/심볼릭 링크, 소스 일치와 압축 해제 후 테스트 프로그램 실행을 검사한다. Windows의 `tests/test_package_zip.ps1`은 실제 zip 안의 MCP/라이선스/소스와 SHA256을 확인한다.

이 검증은 실제 Linux/macOS GUI의 네이티브 빌드·실행, 깨끗한 PC에서의 도구 설치, ARM64, macOS 서명/공증을 대신하지 않는다. 해당 환경의 최종 릴리스 전에 실제 실행을 확인해야 한다.

## 내부 워크스페이스 라이선스 검사

```sh
python script/check_don_licenses.py
# Linux/macOS에서 사용할 수 있는 같은 검사 진입점
bash script/check-don-licenses
```

Python 3.8 이상과 기존 Cargo를 사용하며 새 도구를 설치하지 않는다. `cargo metadata --no-deps --format-version 1 --locked --offline`으로 상속이 해결된 내부 워크스페이스 crate의 SPDX 선언을 읽고, 루트 GPL 전문과 GramEditor/Zed 원저작 고지가 보존됐는지 확인한다. 허용하는 선택지는 `GPL-3.0-only`, `GPL-3.0-or-later`, `MIT`, `Apache-2.0`이며 AND/OR·괄호를 지원한다. 허용 선택지로 충족되지 않는 선언이나 WITH 예외는 별도 검토가 필요하다. 저장한 metadata를 검사하려면 `--metadata <JSON 파일>`을 사용한다.

**외부 Cargo 의존성 전체를 감사하는 도구는 아니다.** keyring 등 신규 의존성은 Cargo.lock의 정확한 소스 아카이브·SHA256, 패키지 license 선언과 LICENSE 전문을 별도로 확인해 `assets/licenses.md`와 `assets/licenses/`에 기록했다. 원본 `script/check-licenses`는 LICENSE 심볼릭 링크를 전제하는 기존 GramEditor 검사기로 보존하며 새 검증기와 구분한다.
