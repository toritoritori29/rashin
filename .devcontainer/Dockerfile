FROM rust:1.71-buster

RUN rustup component add rustfmt

# Setup pyenv
ENV HOME /root
ENV PYENV_ROOT $HOME/.pyenv
ENV PATH $PYENV_ROOT/bin:$PATH
RUN git clone https://github.com/pyenv/pyenv.git $HOME/.pyenv
RUN echo 'eval "$(pyenv init -)"' >> ~/.bashrc
RUN pyenv install 3.11.4
RUN pyenv global 3.11.4

# Setup poetry
ENV PATH $HOME/.local/bin:$PATH
RUN curl -sSL https://install.python-poetry.org | python3 -

# Setup locale
RUN apt-get update && apt-get install -y locales
RUN locale-gen en_US.UTF-8
ENTRYPOINT [ "/bin/bash" ]
